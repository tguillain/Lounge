use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{quick_menu, session, settings};

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                quick_menu::toggle_from_keyboard(app);
            }
        })
        .build()
}

pub fn parse(text: &str) -> Result<Shortcut, String> {
    text.parse::<Shortcut>().map_err(|_| format!("Lounge can't use {text} as a shortcut"))
}

pub fn sync(app: &AppHandle) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    if session::current(app).is_none() {
        return;
    }
    let Ok(path) = settings::path(app) else {
        return;
    };
    if let Some(shortcut) = settings::load(&path).quick_menu_shortcut.as_deref().and_then(|text| parse(text).ok()) {
        let _ = shortcuts.register(shortcut);
    }
}

pub fn change(app: &AppHandle, text: Option<&str>) -> Result<(), String> {
    let shortcut = text.map(parse).transpose()?;
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    if let Some(shortcut) = shortcut {
        let taken = shortcuts.register(shortcut).is_err();
        let _ = shortcuts.unregister_all();
        if taken {
            sync(app);
            return Err("Another app already uses this shortcut. Try a different one.".into());
        }
    }
    settings::set_quick_menu_shortcut(&settings::path(app)?, text.map(str::to_string))?;
    sync(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_shortcuts_the_settings_screen_saves() {
        for text in ["Ctrl+Shift+KeyQ", "Alt+Digit1", "F10", "Shift+Pause", "Ctrl+Numpad5", "Alt+ScrollLock", "Ctrl+Alt+Space"] {
            assert!(parse(text).is_ok(), "{text}");
        }
    }

    #[test]
    fn refuses_text_that_is_not_a_shortcut() {
        assert!(parse("Ctrl+").is_err());
        assert!(parse("Banana").is_err());
    }
}
