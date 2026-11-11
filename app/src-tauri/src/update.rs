use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const AVAILABLE: bool = !cfg!(debug_assertions);
pub const PROGRESS_EVENT: &str = "update-progress";

#[derive(Default)]
pub struct UpdateState {
    pending: Mutex<Option<Update>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    version: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    percent: Option<u8>,
}

pub async fn check(app: &AppHandle) -> Result<Option<AvailableUpdate>, String> {
    if !AVAILABLE {
        return Err("Updates are only checked in the installed app".into());
    }
    let update = app.updater().map_err(|error| error.to_string())?.check().await.map_err(|error| error.to_string())?;
    let available = update.as_ref().map(|update| AvailableUpdate { version: update.version.clone() });
    *app.state::<UpdateState>().pending.lock().map_err(|error| error.to_string())? = update;
    Ok(available)
}

pub async fn install(app: &AppHandle) -> Result<(), String> {
    let update = app
        .state::<UpdateState>()
        .pending
        .lock()
        .map_err(|error| error.to_string())?
        .take()
        .ok_or("No update is ready to install")?;
    let progress_app = app.clone();
    let mut downloaded: u64 = 0;
    let mut last_percent = None;
    update
        .download_and_install(
            move |chunk, total| {
                downloaded += chunk as u64;
                let percent = percent_of(downloaded, total);
                if percent != last_percent {
                    last_percent = percent;
                    let _ = progress_app.emit(PROGRESS_EVENT, Progress { percent });
                }
            },
            || {},
        )
        .await
        .map_err(|error| error.to_string())
}

fn percent_of(downloaded: u64, total: Option<u64>) -> Option<u8> {
    let total = total.filter(|total| *total > 0)?;
    Some((downloaded.min(total) * 100 / total) as u8)
}

#[cfg(test)]
mod tests {
    use super::percent_of;

    #[test]
    fn reports_download_progress_as_a_whole_percentage() {
        assert_eq!(percent_of(0, Some(200)), Some(0));
        assert_eq!(percent_of(99, Some(200)), Some(49));
        assert_eq!(percent_of(250, Some(200)), Some(100));
    }

    #[test]
    fn has_no_percentage_when_the_size_is_unknown() {
        assert_eq!(percent_of(10, None), None);
        assert_eq!(percent_of(10, Some(0)), None);
    }
}
