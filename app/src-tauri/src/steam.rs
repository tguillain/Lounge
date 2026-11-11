use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::{Game, GameImages, Store};
use crate::{logo, vdf};

const FULLY_INSTALLED_FLAG: u32 = 4;
const EXCLUDED_APP_IDS: [&str; 1] = ["228980"];
const CDN_BASE: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub app_id: String,
    pub name: String,
    pub installed: bool,
    pub last_played: u64,
    pub size_on_disk: u64,
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

pub fn scan(logo_cache: Option<&Path>) -> Vec<Game> {
    let Some(root) = steam_root() else {
        return Vec::new();
    };
    let cache = image_cache_dir(&root);
    let mut seen_app_ids = HashSet::new();
    library_folders(&root)
        .iter()
        .flat_map(|library| read_manifests(&library.join("steamapps")))
        .filter(|manifest| manifest.installed && !EXCLUDED_APP_IDS.contains(&manifest.app_id.as_str()))
        .filter(|manifest| seen_app_ids.insert(manifest.app_id.clone()))
        .map(|manifest| to_game(manifest, &cache, logo_cache))
        .collect()
}

pub fn parse_manifest(content: &str) -> Option<Manifest> {
    let doc = vdf::parse(content)?;
    let state = doc.get("AppState")?;
    let flags: u32 = state.text("StateFlags").and_then(|v| v.parse().ok()).unwrap_or(0);
    Some(Manifest {
        app_id: state.text("appid")?.to_string(),
        name: state.text("name")?.to_string(),
        installed: flags & FULLY_INSTALLED_FLAG != 0,
        last_played: state.text("LastPlayed").and_then(|v| v.parse().ok()).unwrap_or(0),
        size_on_disk: state.text("SizeOnDisk").and_then(|v| v.parse().ok()).unwrap_or(0),
    })
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

fn to_game(manifest: Manifest, cache: &Path, logo_cache: Option<&Path>) -> Game {
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
        hidden: false,
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
                installed: true,
                last_played: 1_762_800_000,
                size_on_disk: 21_474_836_480,
            }
        );
    }

    #[test]
    fn flags_an_update_in_progress_as_not_installed() {
        let manifest = parse_manifest(r#""AppState" { "appid" "1" "name" "X" "StateFlags" "1026" }"#).unwrap();
        assert!(!manifest.installed);
    }

    #[test]
    #[ignore = "lit la vraie installation Steam de ce PC"]
    fn prints_local_library() {
        let logo_cache = std::env::temp_dir().join("game-launcher-logos");
        fs::create_dir_all(&logo_cache).unwrap();
        for game in scan(Some(&logo_cache)) {
            println!("{} | {} | dark={} | logo={:?}", game.id, game.name, game.images.logo_is_dark, game.images.logo);
        }
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
