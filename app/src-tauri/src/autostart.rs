use std::path::Path;

pub const AVAILABLE: bool = !cfg!(debug_assertions);

pub fn command_line(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

#[cfg(windows)]
mod platform {
    use std::io::ErrorKind;

    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "Lounge";

    pub fn is_enabled() -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(RUN_KEY, KEY_READ)
            .and_then(|key| key.get_value::<String, _>(VALUE_NAME))
            .is_ok()
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE)
            .map_err(|error| error.to_string())?;
        if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            return key.set_value(VALUE_NAME, &super::command_line(&exe)).map_err(|error| error.to_string());
        }
        match key.delete_value(VALUE_NAME) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error.to_string()),
            _ => Ok(()),
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn is_enabled() -> bool {
        false
    }

    pub fn set_enabled(_enabled: bool) -> Result<(), String> {
        Err("Starting with the system is only supported on Windows".into())
    }
}

pub fn is_enabled() -> bool {
    platform::is_enabled()
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    if !AVAILABLE {
        return Err("Only available in the installed app".into());
    }
    platform::set_enabled(enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_the_exe_path_so_spaces_survive() {
        assert_eq!(
            command_line(Path::new(r"C:\Users\me\AppData\Local\Lounge\lounge.exe")),
            r#""C:\Users\me\AppData\Local\Lounge\lounge.exe""#
        );
    }
}
