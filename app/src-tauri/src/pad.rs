use std::thread;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};
use tauri::AppHandle;

use crate::quick_menu;

const SONY_VENDOR_ID: u16 = 0x054C;
const DUALSENSE_PRODUCT_IDS: [u16; 2] = [0x0CE6, 0x0DF2];
const RETRY_DELAY: Duration = Duration::from_secs(3);
const READ_TIMEOUT_MS: i32 = 1000;
const STICK_UP_THRESHOLD: u8 = 48;
const STICK_DOWN_THRESHOLD: u8 = 208;

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
    thread::spawn(move || loop {
        if let Ok(mut api) = HidApi::new() {
            loop {
                if let Some(device) = open_dualsense(&api) {
                    read_until_disconnected(&app, &device);
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

fn read_until_disconnected(app: &AppHandle, device: &HidDevice) {
    let mut buffer = [0u8; 128];
    let mut previous = Buttons::default();
    loop {
        match device.read_timeout(&mut buffer, READ_TIMEOUT_MS) {
            Ok(0) => continue,
            Ok(length) => {
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
            println!("report 0x{:02x} ({length} bytes) -> {:?}", buffer[0], parse_report(&buffer[..length]));
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
    fn reports_only_new_presses() {
        let held = Buttons { ps: true, cross: true, ..Buttons::default() };
        let next = Buttons { ps: true, cross: false, circle: true, ..Buttons::default() };
        assert_eq!(newly_pressed(held, next), vec![PadButton::Circle]);
    }
}
