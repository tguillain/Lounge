use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};

use crate::{focus, steam};

#[derive(Debug, Clone, Copy)]
pub struct Session {
    pub app_id: u32,
    pub started_at_ms: u64,
}

#[derive(Default)]
pub struct SessionState(Mutex<Option<Session>>);

pub fn start(app: &AppHandle, app_id: u32) {
    let started_at_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0);
    *lock(app) = Some(Session { app_id, started_at_ms });
}

pub fn end(app: &AppHandle) {
    *lock(app) = None;
}

pub fn current(app: &AppHandle) -> Option<Session> {
    *lock(app)
}

pub fn kill_game(app_id: u32) -> Result<usize, String> {
    let install_dir = steam::install_path(app_id).ok_or("Couldn't find where this game is installed")?;
    let system = running_processes();
    let killed = system
        .processes()
        .values()
        .filter(|process| process.exe().is_some_and(|exe| is_inside(exe, &install_dir)))
        .filter(|process| process.kill())
        .count();
    if killed == 0 {
        return Err("No running process found for this game".into());
    }
    Ok(killed)
}

pub fn focus_game(app_id: u32) -> bool {
    let Some(install_dir) = steam::install_path(app_id) else {
        return false;
    };
    let system = running_processes();
    let process_ids: Vec<u32> = system
        .processes()
        .iter()
        .filter(|(_, process)| process.exe().is_some_and(|exe| is_inside(exe, &install_dir)))
        .map(|(pid, _)| pid.as_u32())
        .collect();
    focus::bring_to_front(&process_ids)
}

fn running_processes() -> System {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
}

fn is_inside(path: &Path, folder: &Path) -> bool {
    let normalize = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
    let folder = normalize(folder);
    let folder = folder.trim_end_matches('\\');
    normalize(path).starts_with(&format!("{folder}\\"))
}

fn lock(app: &AppHandle) -> std::sync::MutexGuard<'_, Option<Session>> {
    app.state::<SessionState>().inner().0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_executables_inside_the_install_folder() {
        let folder = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive");
        assert!(is_inside(Path::new(r"c:\program files (x86)\steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe"), folder));
    }

    #[test]
    fn ignores_a_sibling_folder_with_the_same_prefix() {
        let folder = Path::new(r"D:\SteamLibrary\steamapps\common\Game");
        assert!(!is_inside(Path::new(r"D:\SteamLibrary\steamapps\common\Game 2\game.exe"), folder));
    }
}
