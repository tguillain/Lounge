use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewWindowBuilder};

pub const MAIN_LABEL: &str = "main";

#[derive(Default)]
pub struct WindowState {
    closed_for_game: AtomicBool,
}

impl WindowState {
    pub fn closed_for_game(&self) -> bool {
        self.closed_for_game.load(Ordering::SeqCst)
    }
}

pub fn close_for_game(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_LABEL) else {
        return;
    };
    app.state::<WindowState>().closed_for_game.store(true, Ordering::SeqCst);
    let _ = window.destroy();
}

pub fn open_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(config) = app.config().app.windows.iter().find(|window| window.label == MAIN_LABEL).cloned() else {
            return;
        };
        if let Ok(window) = WebviewWindowBuilder::from_config(&app, &config).and_then(|builder| builder.build()) {
            let _ = window.set_focus();
        }
        app.state::<WindowState>().closed_for_game.store(false, Ordering::SeqCst);
    });
}
