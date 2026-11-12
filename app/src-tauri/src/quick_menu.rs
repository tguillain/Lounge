use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::pad::PadButton;
use crate::session;

pub const LABEL: &str = "quickmenu";
const PANEL_WIDTH: f64 = 460.0;

pub fn handle_button(app: &AppHandle, button: PadButton) {
    if button == PadButton::Ps {
        if session::current(app).is_some() {
            toggle(app);
        } else if let Some(main) = app.get_webview_window(crate::window::MAIN_LABEL)
            && main.is_focused().unwrap_or(false)
        {
            let _ = app.emit_to(crate::window::MAIN_LABEL, "power-button", ());
        }
        return;
    }
    if app.get_webview_window(LABEL).is_some() {
        let _ = app.emit_to(LABEL, "pad-action", action_name(button));
    }
}

pub fn close(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.destroy();
    }
}

fn toggle(app: &AppHandle) {
    if app.get_webview_window(LABEL).is_some() {
        close(app);
    } else {
        open(app);
    }
}

fn open(app: &AppHandle) {
    let monitor = app.primary_monitor().ok().flatten();
    let (x, y, height) = monitor.map_or((0.0, 0.0, 900.0), |monitor| {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let position = monitor.position().to_logical::<f64>(scale);
        (position.x + size.width - PANEL_WIDTH, position.y, size.height)
    });
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("quick-menu".into()))
        .title("Lounge")
        .additional_browser_args(crate::window::BROWSER_ARGS)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .inner_size(PANEL_WIDTH, height)
        .position(x, y)
        .focused(true)
        .build();
    if let Ok(window) = built {
        let _ = window.set_focus();
    }
}

fn action_name(button: PadButton) -> &'static str {
    match button {
        PadButton::Up => "up",
        PadButton::Down => "down",
        PadButton::Left => "left",
        PadButton::Right => "right",
        PadButton::Cross => "confirm",
        PadButton::Circle => "back",
        PadButton::Triangle => "options",
        PadButton::Ps => "menu",
    }
}
