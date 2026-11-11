use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub hidden_games: BTreeSet<String>,
    #[serde(default)]
    pub start_fullscreen: bool,
    #[serde(default = "enabled")]
    pub vibration: bool,
    #[serde(default = "enabled")]
    pub show_friends: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hidden_games: BTreeSet::new(),
            start_fullscreen: false,
            vibration: true,
            show_friends: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Toggle {
    StartFullscreen,
    Vibration,
    ShowFriends,
}

fn enabled() -> bool {
    true
}

pub fn path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|error| error.to_string())
}

pub fn load(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

pub fn set_hidden(path: &Path, game_id: &str, hidden: bool) -> Result<(), String> {
    let mut settings = load(path);
    if hidden {
        settings.hidden_games.insert(game_id.to_string());
    } else {
        settings.hidden_games.remove(game_id);
    }
    save(path, &settings)
}

pub fn set_toggle(path: &Path, toggle: Toggle, enabled: bool) -> Result<(), String> {
    let mut settings = load(path);
    let field = match toggle {
        Toggle::StartFullscreen => &mut settings.start_fullscreen,
        Toggle::Vibration => &mut settings.vibration,
        Toggle::ShowFriends => &mut settings.show_friends,
    };
    *field = enabled;
    save(path, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_settings_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lounge-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("settings.json")
    }

    #[test]
    fn starts_empty_when_no_file_exists() {
        assert_eq!(load(&temp_settings_path("missing")), Settings::default());
    }

    #[test]
    fn remembers_and_forgets_a_hidden_game() {
        let path = temp_settings_path("hide");
        set_hidden(&path, "steam:920490", true).unwrap();
        assert!(load(&path).hidden_games.contains("steam:920490"));
        set_hidden(&path, "steam:920490", false).unwrap();
        assert!(load(&path).hidden_games.is_empty());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn tolerates_a_file_from_an_older_version() {
        let settings: Settings = serde_json::from_str(r#"{ "hiddenGames": ["steam:730"] }"#).unwrap();
        assert!(settings.hidden_games.contains("steam:730"));
        assert!(!settings.start_fullscreen);
        assert!(settings.vibration && settings.show_friends);
    }

    #[test]
    fn changes_one_toggle_and_keeps_everything_else() {
        let path = temp_settings_path("toggles");
        set_hidden(&path, "steam:730", true).unwrap();
        set_toggle(&path, Toggle::StartFullscreen, true).unwrap();
        set_toggle(&path, Toggle::Vibration, false).unwrap();
        let settings = load(&path);
        assert!(settings.start_fullscreen);
        assert!(!settings.vibration);
        assert!(settings.show_friends);
        assert!(settings.hidden_games.contains("steam:730"));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
