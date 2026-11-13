use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use crate::process::{self, Process};
use crate::{epic, focus, steam};

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub game_id: String,
    pub started_at_ms: u64,
}

#[derive(Default)]
pub struct SessionState(Mutex<Option<Session>>);

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0)
}

pub fn start(app: &AppHandle, game_id: &str) {
    *lock(app) = Some(Session { game_id: game_id.to_string(), started_at_ms: now_ms() });
}

pub fn end(app: &AppHandle) -> Option<Session> {
    lock(app).take()
}

pub fn current(app: &AppHandle) -> Option<Session> {
    lock(app).clone()
}

pub fn steam_app_id(game_id: &str) -> Option<u32> {
    game_id.strip_prefix("steam:")?.parse().ok()
}

pub fn install_dir(game_id: &str) -> Option<PathBuf> {
    match steam_app_id(game_id) {
        Some(app_id) => steam::install_path(app_id),
        None if game_id.starts_with("epic:") => epic::install_path(game_id),
        None => None,
    }
}

pub fn kill_game(game_id: &str) -> Result<usize, String> {
    let install_dir = install_dir(game_id).ok_or("Couldn't find where this game is installed")?;
    let processes = game_processes(&install_dir);
    if processes.is_empty() {
        return Err("No running process found for this game".into());
    }
    let results: Vec<_> = processes.iter().map(|process| process::terminate(process.id)).collect();
    let killed = results.iter().filter(|result| result.is_ok()).count();
    if killed > 0 {
        return Ok(killed);
    }
    if results.iter().any(|result| result.as_ref().is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied)) {
        return Err("This game runs as administrator, so Lounge can't close it. Quit it from the game's own menu.".into());
    }
    Err("Couldn't close the game".into())
}

pub fn focus_game(game_id: &str) -> bool {
    let Some(install_dir) = install_dir(game_id) else {
        return false;
    };
    let process_ids: Vec<u32> = game_processes(&install_dir).iter().map(|process| process.id).collect();
    focus::bring_to_front(&process_ids)
}

fn game_processes(install_dir: &Path) -> Vec<Process> {
    process::running().into_iter().filter(|process| is_inside(&process.exe, install_dir)).collect()
}

pub fn is_inside(path: &Path, folder: &Path) -> bool {
    let normalize = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
    let folder = normalize(folder);
    let folder = folder.trim_end_matches('\\');
    !folder.is_empty() && normalize(path).starts_with(&format!("{folder}\\"))
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

    #[test]
    fn never_matches_an_empty_folder() {
        assert!(!is_inside(Path::new(r"C:\Windows\explorer.exe"), Path::new("")));
    }

    #[test]
    fn reads_the_steam_app_id_of_a_game() {
        assert_eq!(steam_app_id("steam:730"), Some(730));
        assert_eq!(steam_app_id("epic:Sugar"), None);
    }
}
