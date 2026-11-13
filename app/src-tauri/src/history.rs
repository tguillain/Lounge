//! Every game session Lounge sees, Steam and Epic alike, kept so the Stats
//! tab can show time per day and per week. Steam only keeps totals on the PC,
//! so this history starts the day Lounge starts counting.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const SHORTEST_SESSION_SECONDS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    pub game_id: String,
    /// Unix seconds.
    pub start: u64,
    /// Unix seconds.
    pub end: u64,
}

pub fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("sessions.json"))
}

pub fn load(path: &Path) -> Vec<PlaySession> {
    fs::read_to_string(path).ok().and_then(|content| serde_json::from_str(&content).ok()).unwrap_or_default()
}

/// Adds a finished session. A launch that ends within a minute (a crash, a
/// launcher that bounces) isn't play time and is left out.
pub fn record(path: &Path, game_id: &str, started_ms: u64, ended_ms: u64) -> Result<(), String> {
    let (start, end) = (started_ms / 1000, ended_ms / 1000);
    if end < start + SHORTEST_SESSION_SECONDS {
        return Ok(());
    }
    let mut sessions = load(path);
    sessions.push(PlaySession { game_id: game_id.to_string(), start, end });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string(&sessions).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lounge-history-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("sessions.json")
    }

    #[test]
    fn keeps_every_session_in_order() {
        let path = temp_path("order");
        record(&path, "steam:730", 1_000_000, 1_000_000 + 45 * 60_000).unwrap();
        record(&path, "epic:abc", 2_000_000, 2_000_000 + 90 * 60_000).unwrap();
        let sessions = load(&path);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0], PlaySession { game_id: "steam:730".into(), start: 1000, end: 1000 + 45 * 60 });
        assert_eq!(sessions[1].game_id, "epic:abc");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn skips_launches_shorter_than_a_minute() {
        let path = temp_path("short");
        record(&path, "steam:730", 1_000_000, 1_000_000 + 30_000).unwrap();
        assert!(load(&path).is_empty());
    }

    #[test]
    fn starts_empty_without_a_file() {
        assert!(load(&temp_path("missing")).is_empty());
    }
}
