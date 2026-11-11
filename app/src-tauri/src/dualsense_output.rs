pub const COMMON_LENGTH: usize = 47;

const USB_REPORT_ID: u8 = 0x02;
const USB_LENGTH: usize = 1 + COMMON_LENGTH;
const BLUETOOTH_REPORT_ID: u8 = 0x31;
const BLUETOOTH_LENGTH: usize = 78;
const BLUETOOTH_TAG: u8 = 0x10;
const BLUETOOTH_COMMON_OFFSET: usize = 3;
const BLUETOOTH_CRC_OFFSET: usize = BLUETOOTH_LENGTH - 4;
const BLUETOOTH_CRC_SEED: u8 = 0xA2;

const VALID_FLAG0: usize = 0;
const VALID_FLAG1: usize = 1;
const MOTOR_RIGHT: usize = 2;
const VALID_FLAG2: usize = 38;
const LIGHTBAR_SETUP: usize = 41;
const LIGHTBAR_RED: usize = 44;

const RUMBLE_FLAGS: u8 = 0x03;
const LIGHTBAR_CONTROL: u8 = 0x04;
const RELEASE_LEDS: u8 = 0x08;
const LIGHTBAR_SETUP_CONTROL: u8 = 0x02;
const LIGHTBAR_LIGHT_OUT: u8 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Usb,
    Bluetooth,
}

pub type Common = [u8; COMMON_LENGTH];

pub fn rumble(strength: u8) -> Common {
    let mut common = [0u8; COMMON_LENGTH];
    common[VALID_FLAG0] = RUMBLE_FLAGS;
    common[MOTOR_RIGHT] = strength;
    common
}

pub fn lightbar([red, green, blue]: [u8; 3]) -> Common {
    let mut common = [0u8; COMMON_LENGTH];
    common[VALID_FLAG1] = LIGHTBAR_CONTROL;
    common[LIGHTBAR_RED..LIGHTBAR_RED + 3].copy_from_slice(&[red, green, blue]);
    common
}

pub fn stop_startup_glow() -> Common {
    let mut common = [0u8; COMMON_LENGTH];
    common[VALID_FLAG2] = LIGHTBAR_SETUP_CONTROL;
    common[LIGHTBAR_SETUP] = LIGHTBAR_LIGHT_OUT;
    common
}

pub fn release_lights() -> Common {
    let mut common = [0u8; COMMON_LENGTH];
    common[VALID_FLAG1] = RELEASE_LEDS;
    common
}

pub fn report(transport: Transport, sequence: u8, common: &Common) -> Vec<u8> {
    match transport {
        Transport::Usb => {
            let mut report = vec![0u8; USB_LENGTH];
            report[0] = USB_REPORT_ID;
            report[1..].copy_from_slice(common);
            report
        }
        Transport::Bluetooth => {
            let mut report = vec![0u8; BLUETOOTH_LENGTH];
            report[0] = BLUETOOTH_REPORT_ID;
            report[1] = (sequence & 0x0F) << 4;
            report[2] = BLUETOOTH_TAG;
            report[BLUETOOTH_COMMON_OFFSET..BLUETOOTH_COMMON_OFFSET + COMMON_LENGTH].copy_from_slice(common);
            let crc = crc32(std::iter::once(BLUETOOTH_CRC_SEED).chain(report[..BLUETOOTH_CRC_OFFSET].iter().copied()));
            report[BLUETOOTH_CRC_OFFSET..].copy_from_slice(&crc.to_le_bytes());
            report
        }
    }
}

fn crc32(bytes: impl Iterator<Item = u8>) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_usb_rumble_report_for_the_weak_motor() {
        let usb = report(Transport::Usb, 0, &rumble(0x70));
        assert_eq!(usb.len(), 48);
        assert_eq!(&usb[..5], &[0x02, 0x03, 0x00, 0x70, 0x00]);
        assert!(usb[5..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn places_the_lightbar_color_at_the_end_of_a_usb_report() {
        let usb = report(Transport::Usb, 0, &lightbar([10, 20, 30]));
        assert_eq!(usb[2], LIGHTBAR_CONTROL);
        assert_eq!(&usb[45..48], &[10, 20, 30]);
    }

    #[test]
    fn wraps_bluetooth_reports_with_a_sequence_and_checksum() {
        let bluetooth = report(Transport::Bluetooth, 3, &lightbar([10, 20, 30]));
        assert_eq!(bluetooth.len(), 78);
        assert_eq!(&bluetooth[..3], &[0x31, 0x30, 0x10]);
        assert_eq!(&bluetooth[47..50], &[10, 20, 30]);
        let expected = crc32(std::iter::once(0xA2).chain(bluetooth[..74].iter().copied()));
        assert_eq!(u32::from_le_bytes(bluetooth[74..].try_into().unwrap()), expected);
    }

    #[test]
    fn computes_the_standard_crc32() {
        assert_eq!(crc32(b"123456789".iter().copied()), 0xCBF4_3926);
    }
}
