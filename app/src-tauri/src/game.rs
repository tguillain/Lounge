use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Steam,
    Epic,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameImages {
    pub hero: Option<String>,
    pub logo: Option<String>,
    pub logo_is_dark: bool,
    pub header: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadKind {
    Install,
    Update,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    Downloading,
    Paused,
    Queued,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub kind: DownloadKind,
    pub state: DownloadState,
    pub percent: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub store: Store,
    pub name: String,
    pub last_played: u64,
    pub size_on_disk: u64,
    pub play_time: Option<u64>,
    pub launch_uri: String,
    pub images: GameImages,
    pub hidden: bool,
    pub download: Option<Download>,
}
