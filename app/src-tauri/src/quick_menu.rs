use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::pad::PadButton;
use crate::session;

pub const LABEL: &str = "quickmenu";
pub const OPEN_SETTINGS_EVENT: &str = "open-settings";
const PANEL_WIDTH: f64 = 440.0;
const MARGIN: f64 = 24.0;

#[derive(Default)]
pub struct SettingsRequest(AtomicBool);

impl SettingsRequest {
    pub fn ask(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn take(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}

#[derive(Debug, PartialEq)]
struct Placement {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn placement(area_x: f64, area_y: f64, area_width: f64, area_height: f64) -> Placement {
    let width = PANEL_WIDTH + MARGIN * 2.0;
    Placement { x: area_x + area_width - width, y: area_y, width, height: area_height }
}

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

pub fn toggle_from_keyboard(app: &AppHandle) {
    if session::current(app).is_some() {
        toggle(app);
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
    let spot = monitor.map_or_else(
        || placement(0.0, 0.0, 1600.0, 900.0),
        |monitor| {
            let scale = monitor.scale_factor();
            let area = monitor.work_area();
            let size = area.size.to_logical::<f64>(scale);
            let position = area.position.to_logical::<f64>(scale);
            placement(position.x, position.y, size.width, size.height)
        },
    );
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("quick-menu".into()))
        .title("Lounge")
        .additional_browser_args(crate::window::BROWSER_ARGS)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .inner_size(spot.width, spot.height)
        .position(spot.x, spot.y)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sits_against_the_right_of_the_usable_area_with_room_for_the_margins() {
        let spot = placement(0.0, 0.0, 1920.0, 1032.0);
        assert_eq!(spot, Placement { x: 1432.0, y: 0.0, width: 488.0, height: 1032.0 });
    }

    #[test]
    fn follows_a_screen_placed_left_of_the_main_one() {
        assert_eq!(placement(-2560.0, 40.0, 2560.0, 1400.0).x, -488.0);
    }

    #[test]
    fn hands_a_settings_request_over_once() {
        let request = SettingsRequest::default();
        assert!(!request.take());
        request.ask();
        assert!(request.take());
        assert!(!request.take());
    }
}
