use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::settings;

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

pub fn build_main(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == MAIN_LABEL)
        .cloned()
        .ok_or(tauri::Error::WindowNotFound)?;
    let fullscreen = settings::path(app).map(|path| settings::load(&path).start_fullscreen).unwrap_or(false);
    WebviewWindowBuilder::from_config(app, &config)?.fullscreen(fullscreen).build()
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
        if let Ok(window) = build_main(&app) {
            let _ = window.set_focus();
        }
        app.state::<WindowState>().closed_for_game.store(false, Ordering::SeqCst);
    });
}
