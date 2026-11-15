//! Installing a Steam game you own: Lounge can't start a download itself,
//! so it opens Steam's install window, puts it in front of Lounge (even in
//! full screen), and brings Lounge back once that window closes.

use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{focus, process, steam, window};

pub const WINDOW_EVENT: &str = "steam-install-window";
const STEAM_PROCESSES: [&str; 2] = ["steam.exe", "steamwebhelper.exe"];
const POLL: Duration = Duration::from_millis(250);
const FIND_WINDOW_FOR: Duration = Duration::from_secs(15);
const ACCEPT_ANY_WINDOW_AFTER: Duration = Duration::from_secs(4);
const WAIT_FOR_CLOSE: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub path: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct WindowNews {
    game_id: String,
    state: &'static str,
}

pub fn libraries() -> Vec<Library> {
    let Some(root) = steam::steam_root() else {
        return Vec::new();
    };
    steam::library_folders(&root)
        .into_iter()
        .filter(|folder| folder.is_dir())
        .map(|folder| {
            let (free_bytes, total_bytes) = disk_space(&folder).unwrap_or((0, 0));
            Library { path: folder.to_string_lossy().into_owned(), free_bytes, total_bytes }
        })
        .collect()
}

pub fn start(app: &AppHandle, app_id: u32, name: String) -> Result<(), String> {
    let before: HashSet<isize> = steam_windows().into_iter().map(|(window, _)| window).collect();
    focus::allow_any_foreground();
    app.opener()
        .open_url(format!("steam://install/{app_id}"), None::<&str>)
        .map_err(|error| error.to_string())?;
    let app = app.clone();
    let game_id = format!("steam:{app_id}");
    thread::spawn(move || {
        let tell = |state: &'static str| {
            let _ = app.emit(WINDOW_EVENT, WindowNews { game_id: game_id.clone(), state });
        };
        let Some(dialog) = find_dialog(&before, &name) else {
            tell("not-found");
            return;
        };
        focus::raise(dialog);
        tell("opened");
        let opened = Instant::now();
        while focus::is_open(dialog) && opened.elapsed() < WAIT_FOR_CLOSE {
            thread::sleep(POLL * 2);
        }
        bring_lounge_back(&app);
        tell("closed");
    });
    Ok(())
}

fn find_dialog(before: &HashSet<isize>, name: &str) -> Option<isize> {
    let started = Instant::now();
    let wanted = name.to_lowercase();
    while started.elapsed() < FIND_WINDOW_FOR {
        thread::sleep(POLL);
        let fresh: Vec<(isize, String)> = steam_windows().into_iter().filter(|(window, _)| !before.contains(window)).collect();
        if let Some((window, _)) = fresh.iter().find(|(_, title)| title.to_lowercase().contains(&wanted)) {
            return Some(*window);
        }
        if started.elapsed() >= ACCEPT_ANY_WINDOW_AFTER
            && let Some((window, _)) = fresh.iter().find(|(_, title)| title.trim() != "Steam")
        {
            return Some(*window);
        }
    }
    None
}

fn steam_windows() -> Vec<(isize, String)> {
    let ids: Vec<u32> = process::running()
        .into_iter()
        .filter(|running| is_steam_process(&running.exe))
        .map(|running| running.id)
        .collect();
    if ids.is_empty() { Vec::new() } else { focus::visible_windows(&ids) }
}

pub fn is_steam_process(exe: &Path) -> bool {
    exe.file_name()
        .map(|file| file.to_string_lossy().to_lowercase())
        .is_some_and(|file| STEAM_PROCESSES.contains(&file.as_str()))
}

pub fn bring_lounge_back(app: &AppHandle) {
    let Some(main) = app.get_webview_window(window::MAIN_LABEL) else {
        window::open_main(app);
        return;
    };
    let _ = main.unminimize();
    let _ = main.show();
    if let Ok(handle) = main.hwnd() {
        let handle = handle.0 as isize;
        let _ = app.run_on_main_thread(move || {
            focus::raise(handle);
        });
    } else {
        let _ = main.set_focus();
    }
}

#[cfg(windows)]
fn disk_space(folder: &Path) -> Option<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = folder.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let (mut free, mut total, mut total_free) = (0u64, 0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, &mut total_free) };
    (ok != 0).then_some((free, total))
}

#[cfg(not(windows))]
fn disk_space(_folder: &Path) -> Option<(u64, u64)> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_steam_and_its_web_helper() {
        assert!(is_steam_process(Path::new(r"C:\Program Files (x86)\Steam\steam.exe")));
        assert!(is_steam_process(Path::new(r"C:\Program Files (x86)\Steam\bin\cef\cef.win7x64\steamwebhelper.exe")));
        assert!(!is_steam_process(Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Game\game.exe")));
    }

    #[test]
    #[ignore = "reads the Steam libraries of this PC"]
    fn reads_this_pc() {
        for library in libraries() {
            println!("{} · {} GB free of {} GB", library.path, library.free_bytes / 1_000_000_000, library.total_bytes / 1_000_000_000);
        }
    }
}
