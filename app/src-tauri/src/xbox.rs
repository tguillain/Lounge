//! Xbox controllers through XInput. The interface already reads them through
//! the browser while Lounge is on screen; this reader adds what the browser
//! can't: the Xbox button, the Quick Menu over a game, and the battery.

use std::thread;
use std::time::{Duration, Instant};

use tauri::AppHandle;

use crate::pad::{self, Battery, PadButton};
use crate::quick_menu::{self, Family};

const SLOTS: u32 = 4;
const FAST_POLL: Duration = Duration::from_millis(16);
const IDLE_POLL: Duration = Duration::from_millis(1000);
const RESCAN_EVERY: Duration = Duration::from_secs(1);
const BATTERY_EVERY: Duration = Duration::from_secs(10);
const STICK_DEADZONE: i32 = 16_000;
const STEAM_VIRTUAL_VENDOR: u16 = 0x28DE;

const DPAD_UP: u16 = 0x0001;
const DPAD_DOWN: u16 = 0x0002;
const DPAD_LEFT: u16 = 0x0004;
const DPAD_RIGHT: u16 = 0x0008;
const GUIDE: u16 = 0x0400;
const A: u16 = 0x1000;
const B: u16 = 0x2000;
const Y: u16 = 0x8000;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pad {
    pub buttons: u16,
    pub stick_x: i16,
    pub stick_y: i16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    y: bool,
    guide: bool,
}

pub fn held(pad: Pad) -> Held {
    let x = i32::from(pad.stick_x);
    let y = i32::from(pad.stick_y);
    let stick = (x.abs() >= STICK_DEADZONE || y.abs() >= STICK_DEADZONE).then(|| {
        if x.abs() >= y.abs() {
            if x > 0 { PadButton::Right } else { PadButton::Left }
        } else if y > 0 {
            PadButton::Up
        } else {
            PadButton::Down
        }
    });
    let pressed = |mask: u16| pad.buttons & mask != 0;
    Held {
        up: pressed(DPAD_UP) || stick == Some(PadButton::Up),
        down: pressed(DPAD_DOWN) || stick == Some(PadButton::Down),
        left: pressed(DPAD_LEFT) || stick == Some(PadButton::Left),
        right: pressed(DPAD_RIGHT) || stick == Some(PadButton::Right),
        a: pressed(A),
        b: pressed(B),
        y: pressed(Y),
        guide: pressed(GUIDE),
    }
}

/// The Xbox button plays the PS button's part, A confirms, B goes back and Y
/// opens options, as everywhere else in Lounge.
pub fn newly_pressed(previous: Held, current: Held) -> Vec<PadButton> {
    [
        (previous.up, current.up, PadButton::Up),
        (previous.down, current.down, PadButton::Down),
        (previous.left, current.left, PadButton::Left),
        (previous.right, current.right, PadButton::Right),
        (previous.a, current.a, PadButton::Cross),
        (previous.b, current.b, PadButton::Circle),
        (previous.y, current.y, PadButton::Triangle),
        (previous.guide, current.guide, PadButton::Ps),
    ]
    .into_iter()
    .filter(|(was, is, _)| !was && *is)
    .map(|(_, _, button)| button)
    .collect()
}

/// XInput only says empty, low, medium or full.
pub fn battery_of(kind: u8, level: u8) -> Option<Battery> {
    const WIRED: u8 = 1;
    const DISCONNECTED: u8 = 0;
    if kind == DISCONNECTED || kind == WIRED || kind == 0xFF {
        return None;
    }
    let percent = match level {
        0 => 5,
        1 => 35,
        2 => 70,
        _ => 100,
    };
    Some(Battery { percent, charging: false })
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let Some(input) = platform::XInput::load() else {
            return;
        };
        let mut connected: Vec<u32> = Vec::new();
        let mut previous = [Held::default(); SLOTS as usize];
        let mut last_scan = Instant::now() - RESCAN_EVERY;
        let mut last_battery = Instant::now() - BATTERY_EVERY;
        loop {
            if last_scan.elapsed() >= RESCAN_EVERY {
                last_scan = Instant::now();
                connected = (0..SLOTS).filter(|slot| input.is_real_controller(*slot)).collect();
                for slot in 0..SLOTS {
                    if !connected.contains(&slot) {
                        previous[slot as usize] = Held::default();
                    }
                }
            }
            if last_battery.elapsed() >= BATTERY_EVERY {
                last_battery = Instant::now();
                let battery = connected.iter().find_map(|slot| input.battery(*slot));
                pad::publish_xbox_battery(&app, battery);
            }
            for slot in connected.clone() {
                let Some(state) = input.state(slot) else {
                    continue;
                };
                let current = held(state);
                for button in newly_pressed(previous[slot as usize], current) {
                    quick_menu::handle_button(&app, button, Family::Xbox);
                }
                previous[slot as usize] = current;
            }
            thread::sleep(if connected.is_empty() { IDLE_POLL } else { FAST_POLL });
        }
    });
}

#[cfg(windows)]
mod platform {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows_sys::Win32::UI::Input::XboxController::{
        BATTERY_DEVTYPE_GAMEPAD, XINPUT_BATTERY_INFORMATION, XINPUT_CAPABILITIES, XINPUT_STATE, XInputGetBatteryInformation,
        XInputGetState,
    };

    use super::{Pad, STEAM_VIRTUAL_VENDOR};
    use crate::pad::Battery;

    #[repr(C)]
    struct CapabilitiesEx {
        capabilities: XINPUT_CAPABILITIES,
        vendor_id: u16,
        product_id: u16,
        product_version: u16,
        unknown1: u16,
        unknown2: u32,
    }

    type GetStateEx = unsafe extern "system" fn(u32, *mut XINPUT_STATE) -> u32;
    type GetCapabilitiesEx = unsafe extern "system" fn(u32, u32, u32, *mut CapabilitiesEx) -> u32;

    /// The Xbox button is only reported by XInputGetStateEx, which xinput1_4
    /// exports by ordinal (100), like XInputGetCapabilitiesEx (108).
    pub struct XInput {
        get_state_ex: Option<GetStateEx>,
        get_capabilities_ex: Option<GetCapabilitiesEx>,
    }

    impl XInput {
        pub fn load() -> Option<Self> {
            let name: Vec<u16> = "xinput1_4.dll\0".encode_utf16().collect();
            let module = unsafe { LoadLibraryW(name.as_ptr()) };
            if module.is_null() {
                return None;
            }
            let get_state_ex = unsafe { GetProcAddress(module, 100 as *const u8) }
                .map(|function| unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, GetStateEx>(function) });
            let get_capabilities_ex = unsafe { GetProcAddress(module, 108 as *const u8) }
                .map(|function| unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, GetCapabilitiesEx>(function) });
            Some(Self { get_state_ex, get_capabilities_ex })
        }

        fn raw_state(&self, slot: u32) -> Option<XINPUT_STATE> {
            let mut state: XINPUT_STATE = unsafe { std::mem::zeroed() };
            let result = match self.get_state_ex {
                Some(get_state_ex) => unsafe { get_state_ex(slot, &mut state) },
                None => unsafe { XInputGetState(slot, &mut state) },
            };
            (result == ERROR_SUCCESS).then_some(state)
        }

        pub fn state(&self, slot: u32) -> Option<Pad> {
            let state = self.raw_state(slot)?;
            Some(Pad { buttons: state.Gamepad.wButtons, stick_x: state.Gamepad.sThumbLX, stick_y: state.Gamepad.sThumbLY })
        }

        /// A connected controller that isn't one Steam emulates for another
        /// pad (a DualSense through Steam Input, read on its own already).
        pub fn is_real_controller(&self, slot: u32) -> bool {
            if self.raw_state(slot).is_none() {
                return false;
            }
            let Some(get_capabilities_ex) = self.get_capabilities_ex else {
                return true;
            };
            let mut capabilities: CapabilitiesEx = unsafe { std::mem::zeroed() };
            let result = unsafe { get_capabilities_ex(1, slot, 0, &mut capabilities) };
            result != ERROR_SUCCESS || capabilities.vendor_id != STEAM_VIRTUAL_VENDOR
        }

        pub fn battery(&self, slot: u32) -> Option<Battery> {
            let mut information: XINPUT_BATTERY_INFORMATION = unsafe { std::mem::zeroed() };
            let result = unsafe { XInputGetBatteryInformation(slot, BATTERY_DEVTYPE_GAMEPAD, &mut information) };
            if result != ERROR_SUCCESS {
                return None;
            }
            super::battery_of(information.BatteryType as u8, information.BatteryLevel as u8)
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::Pad;
    use crate::pad::Battery;

    pub struct XInput;

    impl XInput {
        pub fn load() -> Option<Self> {
            None
        }

        pub fn state(&self, _slot: u32) -> Option<Pad> {
            None
        }

        pub fn is_real_controller(&self, _slot: u32) -> bool {
            false
        }

        pub fn battery(&self, _slot: u32) -> Option<Battery> {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_xbox_buttons_to_lounge_actions() {
        let pressed = held(Pad { buttons: A | GUIDE | DPAD_DOWN, stick_x: 0, stick_y: 0 });
        assert_eq!(newly_pressed(Held::default(), pressed), vec![PadButton::Down, PadButton::Cross, PadButton::Ps]);
        assert!(newly_pressed(pressed, pressed).is_empty());
    }

    #[test]
    fn reads_the_left_stick_with_up_pointing_up() {
        assert!(held(Pad { buttons: 0, stick_x: 0, stick_y: 30_000 }).up);
        assert!(held(Pad { buttons: 0, stick_x: -30_000, stick_y: 2_000 }).left);
        assert_eq!(held(Pad { buttons: 0, stick_x: 9_000, stick_y: -9_000 }), Held::default());
    }

    #[test]
    fn turns_xinput_battery_levels_into_a_rough_percentage() {
        assert_eq!(battery_of(2, 3), Some(Battery { percent: 100, charging: false }));
        assert_eq!(battery_of(3, 1).map(|battery| battery.percent), Some(35));
        assert_eq!(battery_of(1, 3), None);
        assert_eq!(battery_of(0, 0), None);
    }

    #[test]
    #[ignore = "reads the Xbox controllers connected to this PC"]
    fn reads_this_pc() {
        let Some(input) = platform::XInput::load() else {
            println!("xinput1_4.dll missing");
            return;
        };
        for slot in 0..SLOTS {
            println!("slot {slot}: real={} state={:?} battery={:?}", input.is_real_controller(slot), input.state(slot), input.battery(slot));
        }
    }
}
