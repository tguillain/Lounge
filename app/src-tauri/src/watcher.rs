use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::process::Scanner;
use crate::session::Session;
use crate::{epic, playtime, quick_menu, session, shortcut, steam, window};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
pub const DOWNLOADS_EVENT: &str = "steam-downloads";

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut processes = Scanner::new();
        let mut downloads = steam::downloads();
        let mut previous = running_game(&mut processes);
        if let Some(game_id) = &previous {
            session::start(&app, game_id);
            shortcut::sync(&app);
        }
        loop {
            thread::sleep(POLL_INTERVAL);
            let latest_downloads = steam::downloads();
            if latest_downloads != downloads {
                downloads = latest_downloads;
                let _ = app.emit(DOWNLOADS_EVENT, &downloads);
            }
            let current = running_game(&mut processes);
            if current == previous {
                continue;
            }
            if let Some(ended) = session::end(&app) {
                remember(&app, &ended);
            }
            match &current {
                Some(game_id) => {
                    session::start(&app, game_id);
                    shortcut::sync(&app);
                    let _ = app.emit("game-started", game_id);
                    window::close_for_game(&app);
                }
                None => {
                    shortcut::sync(&app);
                    quick_menu::close(&app);
                    if let Some(ended) = &previous {
                        let _ = app.emit("game-ended", ended);
                    }
                    window::open_main(&app);
                }
            }
            previous = current;
        }
    });
}

fn running_game(processes: &mut Scanner) -> Option<String> {
    let steam_app = steam::running_app_id();
    if steam_app != 0 {
        return Some(format!("steam:{steam_app}"));
    }
    let installed = epic::installed();
    if installed.is_empty() {
        return None;
    }
    let executables: Vec<_> = processes.running().into_iter().map(|process| process.exe).collect();
    epic::running_game(&installed, &executables)
}

fn remember(app: &AppHandle, ended: &Session) {
    if !ended.game_id.starts_with("epic:") {
        return;
    }
    if let Some(path) = playtime::path(app) {
        let _ = playtime::record(&path, &ended.game_id, ended.started_at_ms, session::now_ms());
    }
}
