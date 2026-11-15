#[cfg(windows)]
mod platform {
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE};
    use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        ASFW_ANY, AllowSetForegroundWindow, BringWindowToTop, EnumWindows, GW_OWNER, GetForegroundWindow, GetWindow,
        GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SW_RESTORE, SetForegroundWindow,
        ShowWindow,
    };

    struct Search {
        process_ids: Vec<u32>,
        found: Vec<(HWND, String)>,
        first_only: bool,
    }

    unsafe extern "system" fn visit(window: HWND, context: LPARAM) -> BOOL {
        let search = unsafe { &mut *(context as *mut Search) };
        let mut process_id = 0u32;
        unsafe { GetWindowThreadProcessId(window, &mut process_id) };
        let length = unsafe { GetWindowTextLengthW(window) };
        let is_main_window = unsafe { IsWindowVisible(window) != 0 && GetWindow(window, GW_OWNER).is_null() && length > 0 };
        if search.process_ids.contains(&process_id) && is_main_window {
            let mut buffer = vec![0u16; length as usize + 1];
            let copied = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
            search.found.push((window, String::from_utf16_lossy(&buffer[..copied.max(0) as usize])));
            if search.first_only {
                return FALSE;
            }
        }
        TRUE
    }

    fn windows_of(process_ids: &[u32], first_only: bool) -> Vec<(HWND, String)> {
        let mut search = Search { process_ids: process_ids.to_vec(), found: Vec::new(), first_only };
        unsafe { EnumWindows(Some(visit), &mut search as *mut Search as LPARAM) };
        search.found
    }

    pub fn visible_windows(process_ids: &[u32]) -> Vec<(isize, String)> {
        windows_of(process_ids, false).into_iter().map(|(window, title)| (window as isize, title)).collect()
    }

    pub fn bring_to_front(process_ids: &[u32]) -> bool {
        windows_of(process_ids, true).first().is_some_and(|(window, _)| raise(*window as isize))
    }

    /// Puts a window in front, even when another program has the focus: the
    /// input of the foreground thread is borrowed for the time of the switch.
    pub fn raise(window: isize) -> bool {
        let window = window as HWND;
        unsafe {
            if IsWindow(window) == 0 {
                return false;
            }
            if IsIconic(window) != 0 {
                ShowWindow(window, SW_RESTORE);
            }
            let foreground = GetForegroundWindow();
            let foreground_thread = if foreground.is_null() { 0 } else { GetWindowThreadProcessId(foreground, null_mut()) };
            let current = GetCurrentThreadId();
            let attached = foreground_thread != 0 && foreground_thread != current && AttachThreadInput(current, foreground_thread, TRUE) != 0;
            BringWindowToTop(window);
            let raised = SetForegroundWindow(window) != 0;
            if attached {
                AttachThreadInput(current, foreground_thread, FALSE);
            }
            raised
        }
    }

    pub fn is_open(window: isize) -> bool {
        let window = window as HWND;
        unsafe { IsWindow(window) != 0 && IsWindowVisible(window) != 0 }
    }

    /// Lets the next program that opens a window take the foreground, which
    /// Windows otherwise refuses to anyone but the program the user is using.
    pub fn allow_any_foreground() {
        unsafe { AllowSetForegroundWindow(ASFW_ANY) };
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn visible_windows(_process_ids: &[u32]) -> Vec<(isize, String)> {
        Vec::new()
    }

    pub fn bring_to_front(_process_ids: &[u32]) -> bool {
        false
    }

    pub fn raise(_window: isize) -> bool {
        false
    }

    pub fn is_open(_window: isize) -> bool {
        false
    }

    pub fn allow_any_foreground() {}
}

pub use platform::{allow_any_foreground, bring_to_front, is_open, raise, visible_windows};
