use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::{quick_menu, session, steam, window};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut previous = steam::running_app_id();
        if previous != 0 {
            session::start(&app, previous);
        }
        loop {
            thread::sleep(POLL_INTERVAL);
            let current = steam::running_app_id();
            if current == previous {
                continue;
            }
            if current != 0 {
                session::start(&app, current);
                let _ = app.emit("game-started", format!("steam:{current}"));
                window::close_for_game(&app);
            } else {
                session::end(&app);
                quick_menu::close(&app);
                let _ = app.emit("game-ended", format!("steam:{previous}"));
                window::open_main(&app);
            }
            previous = current;
        }
    });
}
