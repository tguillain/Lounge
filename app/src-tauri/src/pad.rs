use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::{BusType, HidApi, HidDevice};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::dualsense_output::{self, Common, Transport};
use crate::{quick_menu, window};

const SONY_VENDOR_ID: u16 = 0x054C;
const DUALSENSE_PRODUCT_IDS: [u16; 2] = [0x0CE6, 0x0DF2];
const RETRY_DELAY: Duration = Duration::from_secs(3);
const READ_TIMEOUT_MS: i32 = 50;
const STICK_CENTER: i16 = 128;
const STICK_DEADZONE: i16 = 64;
const USB_INPUT_LENGTH: usize = 64;
const PULSE_STRENGTH: u8 = 0x70;
const PULSE_DURATION: Duration = Duration::from_millis(35);
const USB_STATUS_OFFSET: usize = 53;
const BLUETOOTH_REPORT_ID: u8 = 0x31;
const BLUETOOTH_INPUT_LENGTH: usize = 78;
const BLUETOOTH_STATUS_OFFSET: usize = 54;
const CALIBRATION_FEATURE_REPORT: u8 = 0x05;
const CALIBRATION_REPORT_LENGTH: usize = 41;
const FULL_REPORTS_RETRY: Duration = Duration::from_secs(2);
const BLUETOOTH_HID_SERVICE: &str = "00001124-0000-1000-8000-00805f9b34fb";
pub const BATTERY_EVENT: &str = "controller-battery";
pub const HELD_EVENT: &str = "dualsense-held";
pub const CONNECTED_EVENT: &str = "dualsense-connected";

enum PadCommand {
    Pulse,
    Light,
}

pub struct PadControl {
    commands: Sender<PadCommand>,
    battery: Mutex<Option<Battery>>,
    light: Mutex<Option<[u8; 3]>>,
    connected: AtomicBool,
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

    /// Lounge reads this DualSense itself, so the interface ignores what the
    /// browser reports for it: over Bluetooth, the browser stops seeing it
    /// once it sends full reports.
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
}

/// How a report lays out its bytes. Windows pads every Bluetooth report to
/// the longest one, so a short Bluetooth report can be as long as a USB one:
/// only the connection tells them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Usb,
    Bluetooth,
    Short,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    cross: bool,
    circle: bool,
    square: bool,
    triangle: bool,
    l1: bool,
    r1: bool,
    options: bool,
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
    app.manage(PadControl { commands: sender, battery: Mutex::new(None), light: Mutex::new(None), connected: AtomicBool::new(false) });
    thread::spawn(move || loop {
        if let Ok(mut api) = HidApi::new() {
            loop {
                if let Some((device, transport)) = open_dualsense(&api) {
                    publish_connected(&app, true);
                    read_until_disconnected(&app, &device, transport, &commands);
                    publish_battery(&app, None);
                    publish_connected(&app, false);
                }
                thread::sleep(RETRY_DELAY);
                let _ = api.refresh_devices();
            }
        }
        thread::sleep(RETRY_DELAY);
    });
}

fn open_dualsense(api: &HidApi) -> Option<(HidDevice, Option<Transport>)> {
    let info = api
        .device_list()
        .find(|info| info.vendor_id() == SONY_VENDOR_ID && DUALSENSE_PRODUCT_IDS.contains(&info.product_id()))?;
    let transport = device_transport(info.bus_type(), &info.path().to_string_lossy());
    Some((info.open_device(api).ok()?, transport))
}

pub fn device_transport(bus: BusType, path: &str) -> Option<Transport> {
    match bus {
        BusType::Usb => Some(Transport::Usb),
        BusType::Bluetooth => Some(Transport::Bluetooth),
        _ if path.to_lowercase().contains(BLUETOOTH_HID_SERVICE) => Some(Transport::Bluetooth),
        _ => None,
    }
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

fn read_until_disconnected(app: &AppHandle, device: &HidDevice, transport: Option<Transport>, commands: &Receiver<PadCommand>) {
    let mut buffer = [0u8; 128];
    let mut previous = Buttons::default();
    let mut rumble_ends_at: Option<Instant> = None;
    let mut asked_for_full_reports: Option<Instant> = None;
    let mut output = Output { device, transport, sequence: 0, lit: None, startup_glow_stopped: false };
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
                let Some(layout) = layout_of(report, transport) else {
                    continue;
                };
                if output.transport.is_none() {
                    output.transport = transport_of(layout);
                }
                if layout == Layout::Short && asked_for_full_reports.is_none_or(|asked| asked.elapsed() >= FULL_REPORTS_RETRY) {
                    asked_for_full_reports = Some(Instant::now());
                    ask_for_full_reports(device);
                }
                if let Some(battery) = parse_battery(report, layout) {
                    publish_battery(app, Some(battery));
                }
                if let Some(current) = parse_report(report, layout) {
                    for button in newly_pressed(previous, current) {
                        quick_menu::handle_button(app, button);
                    }
                    if held_actions(&current) != held_actions(&previous) {
                        let _ = app.emit_to(window::MAIN_LABEL, HELD_EVENT, held_actions(&current));
                    }
                    previous = current;
                }
            }
            Err(_) => return,
        }
    }
}

fn publish_connected(app: &AppHandle, connected: bool) {
    app.state::<PadControl>().connected.store(connected, Ordering::SeqCst);
    let _ = app.emit(CONNECTED_EVENT, connected);
    if !connected {
        let _ = app.emit_to(window::MAIN_LABEL, HELD_EVENT, Vec::<&str>::new());
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

pub fn layout_of(report: &[u8], transport: Option<Transport>) -> Option<Layout> {
    match (*report.first()?, report.len(), transport) {
        (BLUETOOTH_REPORT_ID, length, _) if length >= BLUETOOTH_INPUT_LENGTH => Some(Layout::Bluetooth),
        (0x01, length, Some(Transport::Bluetooth)) if length >= 10 => Some(Layout::Short),
        (0x01, length, _) if length >= USB_INPUT_LENGTH => Some(Layout::Usb),
        (0x01, length, _) if length >= 10 => Some(Layout::Short),
        _ => None,
    }
}

pub fn parse_battery(report: &[u8], layout: Layout) -> Option<Battery> {
    let status = match layout {
        Layout::Usb => *report.get(USB_STATUS_OFFSET)?,
        Layout::Bluetooth => *report.get(BLUETOOTH_STATUS_OFFSET)?,
        Layout::Short => return None,
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

fn transport_of(layout: Layout) -> Option<Transport> {
    match layout {
        Layout::Usb => Some(Transport::Usb),
        Layout::Bluetooth => Some(Transport::Bluetooth),
        Layout::Short => None,
    }
}

pub fn parse_report(report: &[u8], layout: Layout) -> Option<Buttons> {
    let (buttons_offset, stick_y_offset) = match layout {
        Layout::Usb => (8, 2),
        Layout::Bluetooth => (9, 3),
        Layout::Short => (5, 2),
    };
    if report.len() <= buttons_offset + 2 {
        return None;
    }
    let face = report[buttons_offset];
    let shoulders = report[buttons_offset + 1];
    let dpad = face & 0x0F;
    let x = i16::from(report[stick_y_offset - 1]) - STICK_CENTER;
    let y = i16::from(report[stick_y_offset]) - STICK_CENTER;
    let stick = (x.abs() >= STICK_DEADZONE || y.abs() >= STICK_DEADZONE).then(|| {
        if x.abs() >= y.abs() {
            if x > 0 { PadButton::Right } else { PadButton::Left }
        } else if y > 0 {
            PadButton::Down
        } else {
            PadButton::Up
        }
    });
    Some(Buttons {
        up: matches!(dpad, 7 | 0 | 1) || stick == Some(PadButton::Up),
        down: matches!(dpad, 3..=5) || stick == Some(PadButton::Down),
        left: matches!(dpad, 5..=7) || stick == Some(PadButton::Left),
        right: matches!(dpad, 1..=3) || stick == Some(PadButton::Right),
        cross: face & 0x20 != 0,
        circle: face & 0x40 != 0,
        square: face & 0x10 != 0,
        triangle: face & 0x80 != 0,
        l1: shoulders & 0x01 != 0,
        r1: shoulders & 0x02 != 0,
        options: shoulders & 0x20 != 0,
        ps: report[buttons_offset + 2] & 0x01 != 0,
    })
}

/// The interface's actions for the buttons held right now, named as in
/// gamepad-mapping.ts. The PS button stays with the Quick Menu and Control Center.
pub fn held_actions(buttons: &Buttons) -> Vec<&'static str> {
    [
        (buttons.left, "left"),
        (buttons.right, "right"),
        (buttons.up, "up"),
        (buttons.down, "down"),
        (buttons.cross, "confirm"),
        (buttons.circle, "back"),
        (buttons.square, "search"),
        (buttons.triangle, "options"),
        (buttons.l1, "previousTab"),
        (buttons.r1, "nextTab"),
        (buttons.options, "menu"),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .map(|(_, action)| action)
    .collect()
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
        let Some((device, _)) = open_dualsense(&api) else {
            println!("no DualSense connected");
            return;
        };
        let info = api
            .device_list()
            .find(|info| info.vendor_id() == SONY_VENDOR_ID && DUALSENSE_PRODUCT_IDS.contains(&info.product_id()))
            .unwrap();
        println!("bus: {:?}", info.bus_type());
        let mut buffer = [0u8; 128];
        for round in 0..2 {
            for _ in 0..3 {
                let length = device.read_timeout(&mut buffer, READ_TIMEOUT_MS).unwrap();
                let report = &buffer[..length];
                let tail = report.iter().skip(10).filter(|byte| **byte != 0).count();
                let layout = layout_of(report, device_transport(info.bus_type(), &info.path().to_string_lossy()));
                println!(
                    "report 0x{:02x} ({length} bytes, {tail} non-zero after byte 10) -> {layout:?} {:?} {:?}",
                    buffer[0],
                    layout.and_then(|layout| parse_report(report, layout)),
                    layout.and_then(|layout| parse_battery(report, layout))
                );
            }
            if round == 0 {
                ask_for_full_reports(&device);
                println!("asked for full reports");
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    fn parse(report: &[u8], transport: Option<Transport>) -> Option<Buttons> {
        parse_report(report, layout_of(report, transport)?)
    }

    fn battery(report: &[u8], transport: Option<Transport>) -> Option<Battery> {
        parse_battery(report, layout_of(report, transport)?)
    }

    fn short_report_padded_by_windows() -> Vec<u8> {
        let mut report = vec![0u8; 78];
        report[..10].copy_from_slice(&[0x01, 0x80, 0x80, 0x80, 0x80, DPAD_NONE | 0x20, 0x00, 0x01, 0, 0]);
        report
    }

    #[test]
    fn reads_the_ps_button_over_usb() {
        let buttons = parse(&usb_report(DPAD_NONE, 0x01), Some(Transport::Usb)).unwrap();
        assert!(buttons.ps);
        assert!(!buttons.cross && !buttons.up);
    }

    #[test]
    fn reads_cross_and_dpad_down_over_usb() {
        let buttons = parse(&usb_report(0x20 | 0x04, 0), Some(Transport::Usb)).unwrap();
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
        let buttons = parse(&report, Some(Transport::Bluetooth)).unwrap();
        assert!(buttons.ps && buttons.circle);
    }

    #[test]
    fn treats_the_left_stick_as_up_and_down() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[2] = 10;
        assert!(parse(&report, Some(Transport::Usb)).unwrap().up);
    }

    #[test]
    fn ignores_unknown_reports() {
        assert!(parse(&[0x05, 1, 2], None).is_none());
        assert!(parse(&[], None).is_none());
    }

    #[test]
    fn reads_a_short_bluetooth_report_even_when_windows_pads_it() {
        let report = short_report_padded_by_windows();
        assert_eq!(layout_of(&report, Some(Transport::Bluetooth)), Some(Layout::Short));
        let buttons = parse(&report, Some(Transport::Bluetooth)).unwrap();
        assert!(buttons.cross && buttons.ps);
        assert_eq!(battery(&report, Some(Transport::Bluetooth)), None);
    }

    #[test]
    fn guesses_the_layout_from_the_length_when_the_connection_is_unknown() {
        assert_eq!(layout_of(&usb_report(DPAD_NONE, 0), None), Some(Layout::Usb));
        assert_eq!(layout_of(&[0x31; 78], None), Some(Layout::Bluetooth));
        assert_eq!(layout_of(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08, 0, 0, 0, 0], None), Some(Layout::Short));
        assert_eq!(transport_of(Layout::Short), None);
    }

    #[test]
    fn reads_the_connection_from_the_device() {
        assert_eq!(device_transport(BusType::Usb, ""), Some(Transport::Usb));
        assert_eq!(device_transport(BusType::Bluetooth, ""), Some(Transport::Bluetooth));
        let path = r"\\?\HID#{00001124-0000-1000-8000-00805F9B34FB}_VID&0002054C_PID&0CE6#9&1f2c&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}";
        assert_eq!(device_transport(BusType::Unknown, path), Some(Transport::Bluetooth));
        assert_eq!(device_transport(BusType::Unknown, r"\\?\HID#VID_054C&PID_0CE6&MI_03#8&2d1"), None);
    }

    #[test]
    fn reads_the_battery_over_usb_while_charging() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0x16;
        assert_eq!(battery(&report, Some(Transport::Usb)), Some(Battery { percent: 65, charging: true }));
    }

    #[test]
    fn reads_the_battery_over_bluetooth() {
        let mut report = vec![0u8; 78];
        report[0] = 0x31;
        report[54] = 0x02;
        assert_eq!(battery(&report, Some(Transport::Bluetooth)), Some(Battery { percent: 25, charging: false }));
    }

    #[test]
    fn caps_the_battery_at_full() {
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0x0A;
        assert_eq!(battery(&report, Some(Transport::Usb)).map(|battery| battery.percent), Some(100));
        report[53] = 0x2A;
        assert_eq!(battery(&report, Some(Transport::Usb)), Some(Battery { percent: 100, charging: true }));
    }

    #[test]
    fn has_no_battery_level_in_short_reports_or_on_errors() {
        assert_eq!(battery(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08, 0, 0, 0, 0], None), None);
        let mut report = usb_report(DPAD_NONE, 0);
        report[53] = 0xA5;
        assert_eq!(battery(&report, Some(Transport::Usb)), None);
    }

    #[test]
    fn maps_every_button_to_an_interface_action() {
        let mut report = usb_report(0x10 | 0x80 | 0x02, 0);
        report[9] = 0x01 | 0x02 | 0x20;
        let buttons = parse(&report, Some(Transport::Usb)).unwrap();
        assert_eq!(held_actions(&buttons), ["right", "search", "options", "previousTab", "nextTab", "menu"]);
    }

    #[test]
    fn reads_the_stick_sideways_over_bluetooth() {
        let mut report = vec![0u8; 78];
        report[0] = 0x31;
        report[2..6].fill(STICK_CENTER);
        report[9] = DPAD_NONE;
        report[2] = 230;
        let buttons = parse(&report, Some(Transport::Bluetooth)).unwrap();
        assert_eq!(held_actions(&buttons), ["right"]);
        report[2] = 150;
        assert!(held_actions(&parse(&report, Some(Transport::Bluetooth)).unwrap()).is_empty());
    }

    #[test]
    fn reports_only_new_presses() {
        let held = Buttons { ps: true, cross: true, ..Buttons::default() };
        let next = Buttons { ps: true, cross: false, circle: true, ..Buttons::default() };
        assert_eq!(newly_pressed(held, next), vec![PadButton::Circle]);
    }
}
