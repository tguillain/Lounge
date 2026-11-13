use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const MINUTE_MS: u64 = 60_000;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Played {
    pub minutes: u64,
    pub last_played: u64,
}

pub type PlayLog = BTreeMap<String, Played>;

pub fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("playtime.json"))
}

pub fn load(path: &Path) -> PlayLog {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn record(path: &Path, game_id: &str, started_at_ms: u64, ended_at_ms: u64) -> Result<(), String> {
    let mut log = load(path);
    let entry = log.entry(game_id.to_string()).or_default();
    entry.minutes += ended_at_ms.saturating_sub(started_at_ms).div_ceil(MINUTE_MS);
    entry.last_played = ended_at_ms / 1000;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(&log).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_log(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lounge-playtime-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("playtime.json")
    }

    #[test]
    fn starts_empty() {
        assert!(load(&temp_log("empty")).is_empty());
    }

    #[test]
    fn adds_up_sessions_and_remembers_the_last_one() {
        let path = temp_log("sessions");
        record(&path, "epic:Among", 0, 25 * MINUTE_MS).unwrap();
        record(&path, "epic:Among", 1_000_000, 1_000_000 + 30 * 1000).unwrap();
        let played = load(&path)["epic:Among"];
        assert_eq!(played.minutes, 26);
        assert_eq!(played.last_played, 1_030);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
