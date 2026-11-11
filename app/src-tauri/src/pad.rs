use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::dualsense_output::{self, Common, Transport};
use crate::quick_menu;

const SONY_VENDOR_ID: u16 = 0x054C;
const DUALSENSE_PRODUCT_IDS: [u16; 2] = [0x0CE6, 0x0DF2];
const RETRY_DELAY: Duration = Duration::from_secs(3);
const READ_TIMEOUT_MS: i32 = 50;
const STICK_UP_THRESHOLD: u8 = 48;
const STICK_DOWN_THRESHOLD: u8 = 208;
const USB_INPUT_LENGTH: usize = 64;
const PULSE_STRENGTH: u8 = 0x70;
const PULSE_DURATION: Duration = Duration::from_millis(35);
const USB_STATUS_OFFSET: usize = 53;
const BLUETOOTH_REPORT_ID: u8 = 0x31;
const BLUETOOTH_INPUT_LENGTH: usize = 78;
const BLUETOOTH_STATUS_OFFSET: usize = 54;
const CALIBRATION_FEATURE_REPORT: u8 = 0x05;
const CALIBRATION_REPORT_LENGTH: usize = 41;
pub const BATTERY_EVENT: &str = "controller-battery";

enum PadCommand {
    Pulse,
    Light,
}

pub struct PadControl {
    commands: Sender<PadCommand>,
    battery: Mutex<Option<Battery>>,
    light: Mutex<Option<[u8; 3]>>,
}

impl PadControl {
    pub fn pulse(&self) {
        let _ = self.commands.send(PadCommand::Pulse);
    }

    pub fn set_light(&self, color: Option<[u8; 3]>) {
        if let Ok(mut light) = self.light.lock() {
            *light = color;
        }
        let _ = self.commands.send(PadCommand::Light);
    }

    fn wanted_light(&self) -> Option<[u8; 3]> {
        self.light.lock().ok().and_then(|light| *light)
    }

    pub fn battery(&self) -> Option<Battery> {
        self.battery.lock().ok().and_then(|battery| *battery)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    cross: bool,
    circle: bool,
    triangle: bool,
    ps: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadButton {
    Up,
    Down,
    Left,
    Right,
    Cross,
    Circle,
    Triangle,
    Ps,
}

pub fn spawn(app: AppHandle) {
    let (sender, commands) = mpsc::channel();
    app.manage(PadControl { commands: sender, battery: Mutex::new(None), light: Mutex::new(None) });
    thread::spawn(move || loop {
        if let Ok(mut api) = HidApi::new() {
            loop {
                if let Some(device) = open_dualsense(&api) {
                    read_until_disconnected(&app, &device, &commands);
                    publish_battery(&app, None);
                }
                thread::sleep(RETRY_DELAY);
                let _ = api.refresh_devices();
            }
        }
        thread::sleep(RETRY_DELAY);
    });
}

fn open_dualsense(api: &HidApi) -> Option<HidDevice> {
    api.device_list()
        .find(|info| info.vendor_id() == SONY_VENDOR_ID && DUALSENSE_PRODUCT_IDS.contains(&info.product_id()))
        .and_then(|info| info.open_device(api).ok())
}

struct Output<'a> {
    device: &'a HidDevice,
    transport: Option<Transport>,
    sequence: u8,
    lit: Option<[u8; 3]>,
    startup_glow_stopped: bool,
}

impl Output<'_> {
    fn send(&mut self, common: &Common) -> bool {
        let Some(transport) = self.transport else {
            return false;
        };
        self.sequence = (self.sequence + 1) % 16;
        self.device.write(&dualsense_output::report(transport, self.sequence, common)).is_ok()
    }

    fn show(&mut self, wanted: Option<[u8; 3]>) {
        if self.transport.is_none() || wanted == self.lit {
            return;
        }
        let sent = match wanted {
            Some(color) => {
                if !self.startup_glow_stopped {
                    self.startup_glow_stopped = self.send(&dualsense_output::stop_startup_glow());
                }
                self.send(&dualsense_output::lightbar(color))
            }
            None => self.send(&dualsense_output::release_lights()),
        };
        if sent {
            self.lit = wanted;
        }
    }
}

fn read_until_disconnected(app: &AppHandle, device: &HidDevice, commands: &Receiver<PadCommand>) {
    let mut buffer = [0u8; 128];
    let mut previous = Buttons::default();
    let mut rumble_ends_at: Option<Instant> = None;
    let mut asked_for_full_reports = false;
    let mut output = Output { device, transport: None, sequence: 0, lit: None, startup_glow_stopped: false };
    commands.try_iter().for_each(drop);
    loop {
        let pulse_requested = commands.try_iter().fold(false, |pulse, command| pulse || matches!(command, PadCommand::Pulse));
        let wired = output.transport == Some(Transport::Usb);
        if pulse_requested && wired && output.send(&dualsense_output::rumble(PULSE_STRENGTH)) {
            rumble_ends_at = Some(Instant::now() + PULSE_DURATION);
        }
        if rumble_ends_at.is_some_and(|end| Instant::now() >= end) {
            output.send(&dualsense_output::rumble(0));
            rumble_ends_at = None;
        }
        output.show(app.state::<PadControl>().wanted_light());
        let timeout = rumble_ends_at
            .map(|end| end.saturating_duration_since(Instant::now()).as_millis().clamp(1, READ_TIMEOUT_MS as u128) as i32)
            .unwrap_or(READ_TIMEOUT_MS);
        match device.read_timeout(&mut buffer, timeout) {
            Ok(0) => continue,
            Ok(length) => {
                let report = &buffer[..length];
                if let Some(transport) = transport_of(report) {
                    output.transport = Some(transport);
                }
                if is_short_bluetooth_report(report) && !asked_for_full_reports {
                    asked_for_full_reports = true;
                    ask_for_full_reports(device);
                }
                if let Some(battery) = parse_battery(report) {
                    publish_battery(app, Some(battery));
                }
                if let Some(current) = parse_report(&buffer[..length]) {
                    for button in newly_pressed(previous, current) {
                        quick_menu::handle_button(app, button);
                    }
                    previous = current;
                }
            }
            Err(_) => return,
        }
    }
}

fn publish_battery(app: &AppHandle, battery: Option<Battery>) {
    let pad = app.state::<PadControl>();
    let Ok(mut current) = pad.battery.lock() else {
        return;
    };
    if *current != battery {
        *current = battery;
        let _ = app.emit(BATTERY_EVENT, battery);
    }
}

fn ask_for_full_reports(device: &HidDevice) {
    let mut calibration = [0u8; CALIBRATION_REPORT_LENGTH];
    calibration[0] = CALIBRATION_FEATURE_REPORT;
    let _ = device.get_feature_report(&mut calibration);
}

fn is_short_bluetooth_report(report: &[u8]) -> bool {
    report.first() == Some(&0x01) && report.len() < USB_INPUT_LENGTH
}

pub fn parse_battery(report: &[u8]) -> Option<Battery> {
    let status = match (*report.first()?, report.len()) {
        (0x01, length) if length >= USB_INPUT_LENGTH => report[USB_STATUS_OFFSET],
        (BLUETOOTH_REPORT_ID, length) if length >= BLUETOOTH_INPUT_LENGTH => report[BLUETOOTH_STATUS_OFFSET],
        _ => return None,
    };
    let level = status & 0x0F;
    let percent = (level * 10 + 5).min(100);
    match status >> 4 {
        0x0 => Some(Battery { percent, charging: false }),
        0x1 => Some(Battery { percent, charging: true }),
        0x2 => Some(Battery { percent: 100, charging: true }),
        _ => None,
    }
}

pub fn is_usb_report(report: &[u8]) -> bool {
    report.first() == Some(&0x01) && report.len() >= USB_INPUT_LENGTH
}

fn transport_of(report: &[u8]) -> Option<Transport> {
    if is_usb_report(report) {
        Some(Transport::Usb)
    } else if report.first() == Some(&BLUETOOTH_REPORT_ID) && report.len() >= BLUETOOTH_INPUT_LENGTH {
        Some(Transport::Bluetooth)
    } else {
        None
    }
}

pub fn parse_report(report: &[u8]) -> Option<Buttons> {
    let (buttons_offset, stick_y_offset) = match (*report.first()?, report.len()) {
        (0x01, length) if length >= 64 => (8, 2),
        (0x31, length) if length >= 12 => (9, 3),
        (0x01, length) if length >= 8 => (5, 2),
        _ => return None,
    };
    let face = report[buttons_offset];
    let dpad = face & 0x0F;
    let stick_y = report[stick_y_offset];
    Some(Buttons {
        up: matches!(dpad, 7 | 0 | 1) || stick_y < STICK_UP_THRESHOLD,
        down: matches!(dpad, 3..=5) || stick_y > STICK_DOWN_THRESHOLD,
        left: matches!(dpad, 5..=7),
        right: matches!(dpad, 1..=3),
        cross: face & 0x20 != 0,
        circle: face & 0x40 != 0,
        triangle: face & 0x80 != 0,
        ps: report[buttons_offset + 2] & 0x01 != 0,
    })
}

pub fn newly_pressed(previous: Buttons, current: Buttons) -> Vec<PadButton> {
    [
        (previous.up, current.up, PadButton::Up),
        (previous.down, current.down, PadButton::Down),
        (previous.left, current.left, PadButton::Left),
        (previous.right, current.right, PadButton::Right),
        (previous.cross, current.cross, PadButton::Cross),
        (previous.circle, current.circle, PadButton::Circle),
        (previous.triangle, current.triangle, PadButton::Triangle),
        (previous.ps, current.ps, PadButton::Ps),
    ]
    .into_iter()
    .filter(|(was, is, _)| !was && *is)
    .map(|(_, _, button)| button)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DPAD_NONE: u8 = 0x08;
    const STICK_CENTER: u8 = 0x80;

    fn usb_report(face: u8, system: u8) -> Vec<u8> {
        let mut report = vec![0u8; 64];
        report[0] = 0x01;
        report[1..5].fill(STICK_CENTER);
        report[8] = face;
        report[10] = system;
        report
    }

    #[test]
    #[ignore = "reads a real DualSense connected to this PC"]
    fn prints_live_dualsense_reports() {
        let api = HidApi::new().unwrap();
        let Some(device) = open_dualsense(&api) else {
            println!("no DualSense connected");
            return;
        };
        let mut buffer = [0u8; 128];
        for _ in 0..5 {
            let length = device.read_timeout(&mut buffer, READ_TIMEOUT_MS).unwrap();
            let report = &buffer[..length];
            println!("report 0x{:02x} ({length} bytes) -> {:?} {:?}", buffer[0], parse_report(report), parse_battery(report));
        }
    }

    #[test]
    fn reads_the_ps_button_over_usb() {
        let buttons = parse_report(&usb_report(DPAD_NONE, 0x01)).unwrap();
        assert!(buttons.ps);
        assert!(!buttons.cross && !buttons.up);
    }

    #[test]
    fn reads_cross_and_dpad_down_over_usb() {
        let buttons = parse_report(&usb_report(0x20 | 0x04, 0)).unwrap();
        assert!(buttons.cross && buttons.down);
        assert!(!buttons.up && !buttons.ps);
    }

    #[test]
    fn reads_the_ps_button_over_bluetooth() {
        let mut report = vec![0u8; 78];
        report[0] = 0x31;
        report[2..6].fill(STICK_CENTER);
        report[9] = DPAD_NONE | 0x40;
        report[11] = 0x01;
        let buttons = parse_report(&report).unwrap();
        assert!(buttons.ps && buttons.circle);
    }

    #[test]
    fn treats_the_left_stick_as_up_and_down() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[2] = 10;
        assert!(parse_report(&report).unwrap().up);
    }

    #[test]
    fn ignores_unknown_reports() {
        assert!(parse_report(&[0x05, 1, 2]).is_none());
        assert!(parse_report(&[]).is_none());
    }

    #[test]
    fn tells_usb_reports_from_bluetooth_ones() {
        assert!(is_usb_report(&usb_report(DPAD_NONE, 0)));
        assert!(!is_usb_report(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08, 0, 0, 0, 0]));
        assert!(!is_usb_report(&[0x31; 78]));
    }

    #[test]
    fn tells_the_connection_from_full_reports_only() {
        assert_eq!(transport_of(&usb_report(DPAD_NONE, 0)), Some(Transport::Usb));
        assert_eq!(transport_of(&[0x31; 78]), Some(Transport::Bluetooth));
        assert_eq!(transport_of(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08, 0, 0, 0, 0]), None);
    }

    #[test]
    fn reads_the_battery_over_usb_while_charging() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0x16;
        assert_eq!(parse_battery(&report), Some(Battery { percent: 65, charging: true }));
    }

    #[test]
    fn reads_the_battery_over_bluetooth() {
        let mut report = vec![0u8; 78];
        report[0] = 0x31;
        report[54] = 0x02;
        assert_eq!(parse_battery(&report), Some(Battery { percent: 25, charging: false }));
    }

    #[test]
    fn caps_the_battery_at_full() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0x0A;
        assert_eq!(parse_battery(&report).map(|battery| battery.percent), Some(100));
        report[53] = 0x2A;
        assert_eq!(parse_battery(&report), Some(Battery { percent: 100, charging: true }));
    }

    #[test]
    fn has_no_battery_level_in_short_reports_or_on_errors() {
        assert_eq!(parse_battery(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08, 0, 0, 0, 0]), None);
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0xA5;
        assert_eq!(parse_battery(&report), None);
    }

    #[test]
    fn reports_only_new_presses() {
        let held = Buttons { ps: true, cross: true, ..Buttons::default() };
        let next = Buttons { ps: true, cross: false, circle: true, ..Buttons::default() };
        assert_eq!(newly_pressed(held, next), vec![PadButton::Circle]);
    }
}
