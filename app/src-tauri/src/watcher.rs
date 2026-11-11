use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::{steam, window};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut previous = steam::running_app_id();
        loop {
            thread::sleep(POLL_INTERVAL);
            let current = steam::running_app_id();
            if current == previous {
                continue;
            }
            if current != 0 {
                let _ = app.emit("game-started", format!("steam:{current}"));
                window::close_for_game(&app);
            } else {
                let _ = app.emit("game-ended", format!("steam:{previous}"));
                window::open_main(&app);
            }
            previous = current;
        }
    });
}
