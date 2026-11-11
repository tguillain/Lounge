#[cfg(windows)]
mod platform {
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GetWindow, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SW_RESTORE,
        SetForegroundWindow, ShowWindow,
    };

    struct Search {
        process_ids: Vec<u32>,
        found: HWND,
    }

    unsafe extern "system" fn visit(window: HWND, context: LPARAM) -> BOOL {
        let search = unsafe { &mut *(context as *mut Search) };
        let mut process_id = 0u32;
        unsafe { GetWindowThreadProcessId(window, &mut process_id) };
        let is_main_window = unsafe {
            IsWindowVisible(window) != 0 && GetWindow(window, GW_OWNER).is_null() && GetWindowTextLengthW(window) > 0
        };
        if search.process_ids.contains(&process_id) && is_main_window {
            search.found = window;
            return FALSE;
        }
        TRUE
    }

    pub fn bring_to_front(process_ids: &[u32]) -> bool {
        let mut search = Search { process_ids: process_ids.to_vec(), found: null_mut() };
        unsafe {
            EnumWindows(Some(visit), &mut search as *mut Search as LPARAM);
            if search.found.is_null() {
                return false;
            }
            if IsIconic(search.found) != 0 {
                ShowWindow(search.found, SW_RESTORE);
            }
            SetForegroundWindow(search.found) != 0
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn bring_to_front(_process_ids: &[u32]) -> bool {
        false
    }
}

pub use platform::bring_to_front;
