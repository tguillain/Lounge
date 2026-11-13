use std::path::Path;

const LAST_SHELL_ERROR: isize = 32;

pub fn open(folder: &Path) -> Result<(), String> {
    if !folder.is_dir() {
        return Err(format!("{} doesn't exist anymore.", folder.display()));
    }
    open_with_shell(folder)
}

#[cfg(windows)]
fn open_with_shell(folder: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide: Vec<u16> = folder.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let result = unsafe {
        ShellExecuteW(std::ptr::null_mut(), std::ptr::null(), wide.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL)
    } as isize;
    if result > LAST_SHELL_ERROR {
        Ok(())
    } else {
        Err(format!("Windows couldn't open the folder (error {result})."))
    }
}

#[cfg(not(windows))]
fn open_with_shell(_folder: &Path) -> Result<(), String> {
    Err("Opening a folder only works on Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_a_folder_that_is_gone() {
        let missing = std::env::temp_dir().join("lounge-folder-that-does-not-exist");
        assert!(open(&missing).unwrap_err().contains("doesn't exist"));
    }
}
