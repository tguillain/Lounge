use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::{Download, DownloadKind, DownloadState, Game, GameImages, Store};
use crate::{logo, vdf};

const UPDATE_REQUIRED_FLAG: u32 = 2;
const FULLY_INSTALLED_FLAG: u32 = 4;
const UPDATE_RUNNING_FLAG: u32 = 256;
const UPDATE_PAUSED_FLAG: u32 = 512;
const UPDATE_STARTED_FLAG: u32 = 1024;
const VALIDATING_FLAG: u32 = 1 << 17;
const ADDING_FILES_FLAG: u32 = 1 << 18;
const PREALLOCATING_FLAG: u32 = 1 << 19;
const DOWNLOADING_FLAG: u32 = 1 << 20;
const STAGING_FLAG: u32 = 1 << 21;
const COMMITTING_FLAG: u32 = 1 << 22;
const WORKING_FLAGS: u32 = UPDATE_RUNNING_FLAG
    | VALIDATING_FLAG
    | ADDING_FILES_FLAG
    | PREALLOCATING_FLAG
    | DOWNLOADING_FLAG
    | STAGING_FLAG
    | COMMITTING_FLAG;
const EXCLUDED_APP_IDS: [&str; 1] = ["228980"];
const CDN_BASE: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub app_id: String,
    pub name: String,
    pub install_dir: String,
    pub installed: bool,
    pub last_played: u64,
    pub size_on_disk: u64,
    pub download: Option<Download>,
}

pub fn steam_root() -> Option<PathBuf> {
    registry_steam_path()
        .map(|path| PathBuf::from(path.replace('/', "\\")))
        .filter(|path| path.is_dir())
        .or_else(|| {
            let fallback = PathBuf::from(r"C:\Program Files (x86)\Steam");
            fallback.is_dir().then_some(fallback)
        })
}

pub fn image_cache_dir(root: &Path) -> PathBuf {
    root.join("appcache").join("librarycache")
}

pub fn running_app_id() -> u32 {
    registry_running_app_id().unwrap_or(0)
}

#[cfg(windows)]
pub fn active_account_id() -> Option<u32> {
    steam_registry_key()?
        .open_subkey("ActiveProcess")
        .ok()?
        .get_value::<u32, _>("ActiveUser")
        .ok()
}

#[cfg(not(windows))]
pub fn active_account_id() -> Option<u32> {
    None
}

pub fn scan(logo_cache: Option<&Path>) -> Vec<Game> {
    let Some(root) = steam_root() else {
        return Vec::new();
    };
    let cache = image_cache_dir(&root);
    let play_times = account_play_times(&root);
    let mut seen_app_ids = HashSet::new();
    library_folders(&root)
        .iter()
        .flat_map(|library| read_manifests(&library.join("steamapps")))
        .filter(|manifest| {
            (manifest.installed || manifest.download.is_some()) && !EXCLUDED_APP_IDS.contains(&manifest.app_id.as_str())
        })
        .filter(|manifest| seen_app_ids.insert(manifest.app_id.clone()))
        .map(|manifest| {
            let play_time = play_times.as_ref().map(|times| times.get(&manifest.app_id).copied().unwrap_or(0));
            to_game(manifest, &cache, logo_cache, play_time)
        })
        .collect()
}

pub fn downloads() -> BTreeMap<String, Download> {
    let Some(root) = steam_root() else {
        return BTreeMap::new();
    };
    library_folders(&root)
        .iter()
        .flat_map(|library| read_manifests(&library.join("steamapps")))
        .filter_map(|manifest| Some((format!("steam:{}", manifest.app_id), manifest.download?)))
        .collect()
}

pub fn find_game(app_id: u32, logo_cache: Option<&Path>) -> Option<Game> {
    let id = format!("steam:{app_id}");
    scan(logo_cache).into_iter().find(|game| game.id == id)
}

pub fn install_path(app_id: u32) -> Option<PathBuf> {
    let root = steam_root()?;
    library_folders(&root).into_iter().find_map(|library| {
        let steamapps = library.join("steamapps");
        let manifest = fs::read_to_string(steamapps.join(format!("appmanifest_{app_id}.acf"))).ok()?;
        let manifest = parse_manifest(&manifest)?;
        (!manifest.install_dir.is_empty()).then(|| steamapps.join("common").join(manifest.install_dir))
    })
}

pub fn parse_manifest(content: &str) -> Option<Manifest> {
    let doc = vdf::parse(content)?;
    let state = doc.get("AppState")?;
    let flags: u32 = state.text("StateFlags").and_then(|v| v.parse().ok()).unwrap_or(0);
    Some(Manifest {
        app_id: state.text("appid")?.to_string(),
        name: state.text("name")?.to_string(),
        install_dir: state.text("installdir").unwrap_or_default().to_string(),
        installed: flags & FULLY_INSTALLED_FLAG != 0,
        last_played: state.text("LastPlayed").and_then(|v| v.parse().ok()).unwrap_or(0),
        size_on_disk: state.text("SizeOnDisk").and_then(|v| v.parse().ok()).unwrap_or(0),
        download: download_of(flags, state),
    })
}

fn download_of(flags: u32, state: &vdf::Vdf) -> Option<Download> {
    let bytes = |key: &str| state.text(key).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let state_now = if flags & UPDATE_PAUSED_FLAG != 0 {
        DownloadState::Paused
    } else if flags & WORKING_FLAGS != 0 {
        DownloadState::Downloading
    } else if flags & FULLY_INSTALLED_FLAG == 0 && flags & (UPDATE_REQUIRED_FLAG | UPDATE_STARTED_FLAG) != 0 {
        DownloadState::Queued
    } else {
        return None;
    };
    let has_installed_files = state.get("InstalledDepots").is_some_and(|depots| !depots.entries().is_empty());
    let total = bytes("BytesToDownload") + bytes("BytesToStage");
    let done = bytes("BytesDownloaded") + bytes("BytesStaged");
    Some(Download {
        kind: if has_installed_files { DownloadKind::Update } else { DownloadKind::Install },
        state: state_now,
        percent: (total > 0).then(|| (done.min(total) * 100 / total).min(99) as u8),
    })
}

pub fn parse_play_times(content: &str) -> HashMap<String, u64> {
    let apps = vdf::parse(content).and_then(|doc| {
        doc.get("UserLocalConfigStore")?
            .get("Software")?
            .get("Valve")?
            .get("Steam")?
            .get("apps")
            .map(|apps| apps.entries().to_vec())
    });
    apps.unwrap_or_default()
        .into_iter()
        .filter_map(|(app_id, app)| Some((app_id, app.text("Playtime")?.parse().ok()?)))
        .collect()
}

pub fn parse_library_folders(content: &str) -> Vec<PathBuf> {
    let Some(doc) = vdf::parse(content) else {
        return Vec::new();
    };
    let Some(folders) = doc.get("libraryfolders") else {
        return Vec::new();
    };
    folders
        .entries()
        .iter()
        .filter_map(|(_, folder)| folder.text("path"))
        .map(PathBuf::from)
        .collect()
}

fn library_folders(root: &Path) -> Vec<PathBuf> {
    let listed = fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf"))
        .map(|content| parse_library_folders(&content))
        .unwrap_or_default();
    let mut seen = HashSet::new();
    std::iter::once(root.to_path_buf())
        .chain(listed)
        .filter(|path| seen.insert(folder_key(path)))
        .collect()
}

fn folder_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn account_play_times(root: &Path) -> Option<HashMap<String, u64>> {
    let userdata = root.join("userdata");
    let config_of = |account: &Path| account.join("config").join("localconfig.vdf");
    let active = active_account_id()
        .filter(|id| *id != 0)
        .map(|id| config_of(&userdata.join(id.to_string())))
        .filter(|path| path.is_file());
    let latest = || {
        fs::read_dir(&userdata)
            .ok()?
            .flatten()
            .map(|entry| config_of(&entry.path()))
            .filter_map(|path| Some((fs::metadata(&path).ok()?.modified().ok()?, path)))
            .max_by_key(|(modified, _)| *modified)
            .map(|(_, path)| path)
    };
    let content = fs::read_to_string(active.or_else(latest)?).ok()?;
    Some(parse_play_times(&content))
}

fn read_manifests(steamapps: &Path) -> Vec<Manifest> {
    let Ok(entries) = fs::read_dir(steamapps) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"))
        })
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|content| parse_manifest(&content))
        .collect()
}

fn to_game(manifest: Manifest, cache: &Path, logo_cache: Option<&Path>, play_time: Option<u64>) -> Game {
    let cached = |names: &[&str]| names.iter().find_map(|file| find_cached_image(cache, &manifest.app_id, file));
    let image = |cached_names: &[&str], cdn_name: &str| {
        Some(cached(cached_names).unwrap_or_else(|| format!("{CDN_BASE}/{}/{cdn_name}", manifest.app_id)))
    };
    let prepared_logo = cached(&["logo.png"]).and_then(|source| {
        logo_cache.and_then(|dir| logo::prepare(Path::new(&source), dir, &format!("steam-{}", manifest.app_id)))
    });
    Game {
        id: format!("steam:{}", manifest.app_id),
        store: Store::Steam,
        launch_uri: format!("steam://rungameid/{}", manifest.app_id),
        images: GameImages {
            hero: image(&["library_hero.jpg"], "library_hero.jpg"),
            logo_is_dark: prepared_logo.as_ref().is_some_and(|prepared| prepared.is_dark),
            logo: prepared_logo
                .map(|prepared| prepared.path.to_string_lossy().into_owned())
                .or_else(|| image(&["logo.png"], "logo.png")),
            header: image(&["library_header.jpg", "header.jpg"], "header.jpg"),
            cover: image(&["library_capsule.jpg", "library_600x900.jpg"], "library_600x900.jpg"),
        },
        name: manifest.name,
        last_played: manifest.last_played,
        size_on_disk: manifest.size_on_disk,
        play_time,
        hidden: false,
        download: manifest.download,
    }
}

fn find_cached_image(cache: &Path, app_id: &str, file: &str) -> Option<String> {
    let flat = cache.join(format!("{app_id}_{file}"));
    if flat.is_file() {
        return Some(flat.to_string_lossy().into_owned());
    }
    let folder = cache.join(app_id);
    let direct = folder.join(file);
    if direct.is_file() {
        return Some(direct.to_string_lossy().into_owned());
    }
    fs::read_dir(&folder)
        .ok()?
        .flatten()
        .map(|entry| entry.path().join(file))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
}

#[cfg(windows)]
fn steam_registry_key() -> Option<winreg::RegKey> {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .ok()
}

#[cfg(windows)]
fn registry_steam_path() -> Option<String> {
    steam_registry_key()?.get_value::<String, _>("SteamPath").ok()
}

#[cfg(windows)]
fn registry_running_app_id() -> Option<u32> {
    steam_registry_key()?.get_value::<u32, _>("RunningAppID").ok()
}

#[cfg(windows)]
pub fn language() -> Option<String> {
    steam_registry_key()?.get_value::<String, _>("Language").ok().filter(|language| !language.is_empty())
}

#[cfg(not(windows))]
pub fn language() -> Option<String> {
    None
}

#[cfg(not(windows))]
fn registry_steam_path() -> Option<String> {
    None
}

#[cfg(not(windows))]
fn registry_running_app_id() -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_installed_manifest() {
        let manifest = parse_manifest(
            r#""AppState"
            {
                "appid"       "252950"
                "name"        "Rocket League"
                "installdir"  "rocketleague"
                "StateFlags"  "4"
                "LastPlayed"  "1762800000"
                "SizeOnDisk"  "21474836480"
            }"#,
        )
        .unwrap();
        assert_eq!(
            manifest,
            Manifest {
                app_id: "252950".into(),
                name: "Rocket League".into(),
                install_dir: "rocketleague".into(),
                installed: true,
                last_played: 1_762_800_000,
                size_on_disk: 21_474_836_480,
                download: None,
            }
        );
    }

    #[test]
    fn flags_an_update_in_progress_as_not_installed() {
        let manifest = parse_manifest(r#""AppState" { "appid" "1" "name" "X" "StateFlags" "1026" }"#).unwrap();
        assert!(!manifest.installed);
    }

    #[test]
    fn reads_the_progress_of_a_running_update() {
        let manifest = parse_manifest(
            r#""AppState" {
                "appid" "730" "name" "Counter-Strike 2" "StateFlags" "1282"
                "BytesToDownload" "400" "BytesDownloaded" "100" "BytesToStage" "600" "BytesStaged" "150"
                "InstalledDepots" { "731" { "manifest" "1" } }
            }"#,
        )
        .unwrap();
        assert_eq!(
            manifest.download,
            Some(Download { kind: DownloadKind::Update, state: DownloadState::Downloading, percent: Some(25) })
        );
    }

    #[test]
    fn tells_a_fresh_install_from_an_update() {
        let manifest = parse_manifest(
            r#""AppState" { "appid" "1" "name" "X" "StateFlags" "1026" "BytesToDownload" "10" "BytesDownloaded" "0" }"#,
        )
        .unwrap();
        let download = manifest.download.unwrap();
        assert_eq!(download.kind, DownloadKind::Install);
        assert_eq!(download.state, DownloadState::Queued);
        assert_eq!(download.percent, Some(0));
    }

    #[test]
    fn reads_a_paused_download() {
        let manifest = parse_manifest(r#""AppState" { "appid" "1" "name" "X" "StateFlags" "1542" }"#).unwrap();
        assert_eq!(manifest.download.map(|download| download.state), Some(DownloadState::Paused));
        assert_eq!(manifest.download.and_then(|download| download.percent), None);
    }

    #[test]
    fn ignores_finished_downloads_and_scheduled_updates_of_installed_games() {
        let finished = r#""AppState" { "appid" "1" "name" "X" "StateFlags" "4" "BytesToDownload" "10" "BytesDownloaded" "10" }"#;
        assert_eq!(parse_manifest(finished).unwrap().download, None);
        let scheduled = r#""AppState" { "appid" "1" "name" "X" "StateFlags" "6" }"#;
        assert_eq!(parse_manifest(scheduled).unwrap().download, None);
    }

    #[test]
    #[ignore = "reads the real Steam install on this PC"]
    fn prints_local_library() {
        let logo_cache = std::env::temp_dir().join("lounge-logos");
        fs::create_dir_all(&logo_cache).unwrap();
        for game in scan(Some(&logo_cache)) {
            println!("{} | {} | {:?} min | dark={}", game.id, game.name, game.play_time, game.images.logo_is_dark);
        }
    }

    #[test]
    fn reads_play_time_in_minutes_per_app() {
        let times = parse_play_times(
            r#""UserLocalConfigStore"
            {
                "Software" { "Valve" { "Steam" { "apps"
                {
                    "730" { "LastPlayed" "1762800000" "Playtime" "4210" }
                    "252950" { "cloud" { "last_sync_state" "synchronized" } }
                } } } }
            }"#,
        );
        assert_eq!(times, HashMap::from([("730".to_string(), 4210)]));
    }

    #[test]
    fn lists_every_library_folder() {
        let folders = parse_library_folders(
            r#""libraryfolders"
            {
                "0" { "path" "C:\\Program Files (x86)\\Steam" }
                "1" { "path" "D:\\SteamLibrary" }
            }"#,
        );
        assert_eq!(folders, vec![PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
    }

    #[test]
    fn treats_registry_and_vdf_spellings_as_one_folder() {
        assert_eq!(
            folder_key(Path::new("c:/program files (x86)/steam/")),
            folder_key(Path::new(r"C:\Program Files (x86)\Steam"))
        );
    }
}
