use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::Deserialize;

use crate::game::{Game, GameImages, Store};
use crate::playtime::PlayLog;
use crate::session::is_inside;

#[derive(Debug, Clone, PartialEq)]
struct Install {
    app_name: String,
    namespace: String,
    item_id: String,
    display_name: Option<String>,
    install_location: String,
    install_size: u64,
    is_game: Option<bool>,
    complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemManifest {
    display_name: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    #[serde(default)]
    install_location: String,
    #[serde(default)]
    install_size: u64,
    #[serde(default)]
    app_categories: Vec<String>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete_install: bool,
}

#[derive(Debug, Deserialize)]
struct InstalledItem {
    v4: InstalledItemV4,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledItemV4 {
    state: String,
    dir: String,
    sandbox_id: String,
    item_id: String,
    artifact_id: String,
    #[serde(default)]
    manifest_data: Option<InstalledManifestData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledManifestData {
    #[serde(default)]
    build_size: u64,
}

#[derive(Debug, Deserialize)]
struct CatalogItem {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    categories: Vec<CatalogCategory>,
    #[serde(default, rename = "keyImages")]
    key_images: Vec<KeyImage>,
}

#[derive(Debug, Deserialize)]
struct CatalogCategory {
    path: String,
}

#[derive(Debug, Deserialize)]
struct KeyImage {
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Debug, Clone, Default)]
pub struct CatalogEntry {
    pub title: Option<String>,
    pub is_game: Option<bool>,
    pub images: GameImages,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Installed {
    pub id: String,
    pub install_dir: PathBuf,
}

pub fn scan(play_log: &PlayLog) -> Vec<Game> {
    let catalog = catalog();
    installs().iter().filter_map(|install| to_game(install, &catalog, play_log)).collect()
}

pub fn find_game(game_id: &str, play_log: &PlayLog) -> Option<Game> {
    scan(play_log).into_iter().find(|game| game.id == game_id)
}

pub fn installed() -> Vec<Installed> {
    let catalog = catalog();
    installs()
        .into_iter()
        .filter(|install| is_playable(install, &catalog) && !install.install_location.is_empty())
        .map(|install| Installed { id: game_id(&install.app_name), install_dir: PathBuf::from(install.install_location) })
        .collect()
}

pub fn install_path(game_id: &str) -> Option<PathBuf> {
    installed().into_iter().find(|game| game.id == game_id).map(|game| game.install_dir)
}

fn game_id(app_name: &str) -> String {
    format!("epic:{app_name}")
}

fn is_playable(install: &Install, catalog: &HashMap<String, CatalogEntry>) -> bool {
    let is_game = install
        .is_game
        .or_else(|| catalog.get(&install.item_id).and_then(|entry| entry.is_game))
        .unwrap_or(true);
    is_game && install.complete
}

fn to_game(install: &Install, catalog: &HashMap<String, CatalogEntry>, play_log: &PlayLog) -> Option<Game> {
    if !is_playable(install, catalog) {
        return None;
    }
    let entry = catalog.get(&install.item_id);
    let name = install
        .display_name
        .clone()
        .or_else(|| entry.and_then(|entry| entry.title.clone()))
        .or_else(|| Path::new(&install.install_location).file_name().map(|name| name.to_string_lossy().into_owned()))?;
    let id = game_id(&install.app_name);
    let played = play_log.get(&id).copied();
    Some(Game {
        launch_uri: format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            install.namespace, install.item_id, install.app_name
        ),
        name,
        last_played: played.map_or(0, |played| played.last_played),
        size_on_disk: install.install_size,
        play_time: played.map(|played| played.minutes),
        recent_play_time: None,
        images: entry.map(|entry| entry.images.clone()).unwrap_or_default(),
        hidden: false,
        download: None,
        store: Store::Epic,
        id,
    })
}

fn from_item(content: &str) -> Option<Install> {
    let manifest: ItemManifest = serde_json::from_str(content).ok()?;
    let is_game = (!manifest.app_categories.is_empty()).then(|| manifest.app_categories.iter().any(|category| category == "games"));
    Some(Install {
        app_name: manifest.app_name,
        namespace: manifest.catalog_namespace,
        item_id: manifest.catalog_item_id,
        display_name: Some(manifest.display_name),
        install_location: manifest.install_location,
        install_size: manifest.install_size,
        is_game,
        complete: !manifest.incomplete_install,
    })
}

fn from_installed_item(content: &str) -> Option<Install> {
    let item = serde_json::from_str::<InstalledItem>(content).ok()?.v4;
    Some(Install {
        app_name: item.artifact_id,
        namespace: item.sandbox_id,
        item_id: item.item_id,
        display_name: None,
        install_location: item.dir,
        install_size: item.manifest_data.map_or(0, |data| data.build_size),
        is_game: None,
        complete: item.state.eq_ignore_ascii_case("installed"),
    })
}

fn installs() -> Vec<Install> {
    let mut found: Vec<Install> = read_files(&data_dir().join("Manifests"), "item", from_item);
    for install in read_files(&installed_items_dir(), "egi", from_installed_item) {
        if !found.iter().any(|known| known.app_name == install.app_name) {
            found.push(install);
        }
    }
    found
}

fn read_files(dir: &Path, extension: &str, parse: fn(&str) -> Option<Install>) -> Vec<Install> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(extension)))
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|content| parse(&content))
        .collect()
}

fn catalog() -> HashMap<String, CatalogEntry> {
    fs::read_to_string(data_dir().join("Catalog").join("catcache.bin"))
        .ok()
        .map(|content| parse_catalog(&content))
        .unwrap_or_default()
}

pub fn parse_catalog(content: &str) -> HashMap<String, CatalogEntry> {
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(content.trim()) else {
        return HashMap::new();
    };
    let Ok(items) = serde_json::from_slice::<Vec<CatalogItem>>(&bytes) else {
        return HashMap::new();
    };
    items
        .into_iter()
        .map(|item| {
            let find = |kind: &str| item.key_images.iter().find(|image| image.kind == kind).map(|image| image.url.clone());
            let wide = find("DieselGameBox").or_else(|| find("OfferImageWide"));
            let images = GameImages {
                hero: wide.clone(),
                logo: find("DieselGameBoxLogo"),
                logo_is_dark: false,
                header: wide.map(|url| resized(&url, 920, 518)).or_else(|| find("Thumbnail")),
                cover: find("DieselGameBoxTall").or_else(|| find("OfferImageTall")).map(|url| resized(&url, 600, 800)),
            };
            let is_game = (!item.categories.is_empty()).then(|| item.categories.iter().any(|category| category.path == "games"));
            (item.id, CatalogEntry { title: item.title, is_game, images })
        })
        .collect()
}

fn resized(url: &str, width: u32, height: u32) -> String {
    if url.contains('?') {
        return url.to_string();
    }
    format!("{url}?w={width}&h={height}&resize=1&quality=high")
}

pub fn running_game(installed: &[Installed], executables: &[PathBuf]) -> Option<String> {
    installed
        .iter()
        .find(|game| executables.iter().any(|exe| is_inside(exe, &game.install_dir)))
        .map(|game| game.id.clone())
}

fn program_data() -> PathBuf {
    std::env::var_os("PROGRAMDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
}

fn data_dir() -> PathBuf {
    program_data().join("Epic").join("EpicGamesLauncher").join("Data")
}

fn installed_items_dir() -> PathBuf {
    program_data()
        .join("Epic")
        .join("EpicOnlineServicesShared")
        .join("InstallHelper")
        .join("InstalledItems")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playtime::Played;

    const ROCKET_LEAGUE: &str = r#"{
        "DisplayName": "Rocket League",
        "AppName": "Sugar",
        "CatalogNamespace": "9773aa1aa54f4f7b80e44bef04986cea",
        "CatalogItemId": "530145df28a24424923f5828cc9031a1",
        "InstallLocation": "D:\\Epic Games\\rocketleague",
        "InstallSize": 25000000000,
        "AppCategories": ["public", "games", "applications"],
        "bIsIncompleteInstall": false
    }"#;

    const AMONG_US: &str = r#"{"v4":{"installationId":"89A33EE4F258879E3839CEFED988C507","state":"Installed",
        "dir":"D:\\Epic Games\\AmongUs","metaDir":"D:\\Epic Games\\AmongUs\\.egstore","platform":"Windows",
        "sandboxId":"33956bcb55d4452d8c47e16b94e294bd","itemId":"729a86a5146640a2ace9e8c595414c56",
        "artifactId":"963137e4c29d4c79a81323b8fab03a40","tags":[],
        "manifestData":{"version":"7489","launchCommand":"","launchExe":"Among Us.exe","buildSize":1149685974}}}"#;

    fn among_us_catalog() -> HashMap<String, CatalogEntry> {
        HashMap::from([(
            "729a86a5146640a2ace9e8c595414c56".to_string(),
            CatalogEntry {
                title: Some("Among Us".into()),
                is_game: Some(true),
                images: GameImages { header: Some("https://cdn/wide.jpg".into()), ..GameImages::default() },
            },
        )])
    }

    fn game_from_item(content: &str) -> Option<Game> {
        to_game(&from_item(content)?, &HashMap::new(), &PlayLog::new())
    }

    #[test]
    fn builds_a_silent_launch_link() {
        let game = game_from_item(ROCKET_LEAGUE).unwrap();
        assert_eq!(game.id, "epic:Sugar");
        assert_eq!(
            game.launch_uri,
            "com.epicgames.launcher://apps/9773aa1aa54f4f7b80e44bef04986cea%3A530145df28a24424923f5828cc9031a1%3ASugar?action=launch&silent=true"
        );
    }

    #[test]
    fn skips_incomplete_installs() {
        let partial = ROCKET_LEAGUE.replace("\"bIsIncompleteInstall\": false", "\"bIsIncompleteInstall\": true");
        assert!(game_from_item(&partial).is_none());
    }

    #[test]
    fn skips_non_game_entries() {
        let engine = ROCKET_LEAGUE.replace("\"public\", \"games\", \"applications\"", "\"engines\"");
        assert!(game_from_item(&engine).is_none());
    }

    #[test]
    fn reads_the_new_install_records_and_names_them_from_the_catalog() {
        let install = from_installed_item(AMONG_US).unwrap();
        let game = to_game(&install, &among_us_catalog(), &PlayLog::new()).unwrap();
        assert_eq!(game.id, "epic:963137e4c29d4c79a81323b8fab03a40");
        assert_eq!(game.name, "Among Us");
        assert_eq!(game.size_on_disk, 1_149_685_974);
        assert_eq!(game.images.header.as_deref(), Some("https://cdn/wide.jpg"));
        assert!(game.launch_uri.starts_with(
            "com.epicgames.launcher://apps/33956bcb55d4452d8c47e16b94e294bd%3A729a86a5146640a2ace9e8c595414c56%3A963137e4c29d4c79a81323b8fab03a40"
        ));
    }

    #[test]
    fn falls_back_to_the_folder_name_without_a_catalog_entry() {
        let install = from_installed_item(AMONG_US).unwrap();
        assert_eq!(to_game(&install, &HashMap::new(), &PlayLog::new()).unwrap().name, "AmongUs");
    }

    #[test]
    fn skips_a_new_record_that_is_still_installing_or_not_a_game() {
        let installing = from_installed_item(&AMONG_US.replace("\"Installed\"", "\"Installing\"")).unwrap();
        assert!(to_game(&installing, &among_us_catalog(), &PlayLog::new()).is_none());
        let mut catalog = among_us_catalog();
        catalog.values_mut().for_each(|entry| entry.is_game = Some(false));
        let install = from_installed_item(AMONG_US).unwrap();
        assert!(to_game(&install, &catalog, &PlayLog::new()).is_none());
    }

    #[test]
    fn takes_play_time_from_what_lounge_counted() {
        let mut log = PlayLog::new();
        log.insert("epic:Sugar".into(), Played { minutes: 95, last_played: 1_700_000_000 });
        let game = to_game(&from_item(ROCKET_LEAGUE).unwrap(), &HashMap::new(), &log).unwrap();
        assert_eq!(game.play_time, Some(95));
        assert_eq!(game.last_played, 1_700_000_000);
        assert_eq!(game_from_item(ROCKET_LEAGUE).unwrap().play_time, None);
    }

    #[test]
    fn reads_artwork_names_and_categories_from_the_launcher_catalog() {
        let json = r#"[{"id":"729a86a5","title":"Among Us","categories":[{"path":"games"}],"keyImages":[
            {"type":"DieselGameBox","url":"https://cdn1.epicgames.com/wide.jpg"},
            {"type":"DieselGameBoxTall","url":"https://cdn1.epicgames.com/tall.jpg"},
            {"type":"DieselGameBoxLogo","url":"https://cdn1.epicgames.com/logo.png"}]},
            {"id":"bare","title":"No art","categories":[{"path":"engines"}]}]"#;
        let encoded = base64::engine::general_purpose::STANDARD.encode(json);
        let catalog = parse_catalog(&encoded);
        let among = &catalog["729a86a5"];
        assert_eq!(among.title.as_deref(), Some("Among Us"));
        assert_eq!(among.is_game, Some(true));
        assert_eq!(among.images.hero.as_deref(), Some("https://cdn1.epicgames.com/wide.jpg"));
        assert_eq!(among.images.header.as_deref(), Some("https://cdn1.epicgames.com/wide.jpg?w=920&h=518&resize=1&quality=high"));
        assert_eq!(among.images.cover.as_deref(), Some("https://cdn1.epicgames.com/tall.jpg?w=600&h=800&resize=1&quality=high"));
        assert_eq!(among.images.logo.as_deref(), Some("https://cdn1.epicgames.com/logo.png"));
        assert_eq!(catalog["bare"].is_game, Some(false));
        assert!(parse_catalog("not base64!").is_empty());
    }

    #[test]
    #[ignore = "reads this PC's Epic Games Launcher data"]
    fn reads_this_pc() {
        for game in scan(&PlayLog::new()) {
            println!("{} | {} | {} bytes | {:?}", game.id, game.name, game.size_on_disk, game.images.header);
        }
        println!("installed: {:?}", installed());
        assert!(!catalog().is_empty());
    }

    #[test]
    fn finds_the_epic_game_that_is_running() {
        let installed = [
            Installed { id: "epic:Sugar".into(), install_dir: PathBuf::from(r"D:\Epic Games\rocketleague") },
            Installed { id: "epic:Among".into(), install_dir: PathBuf::from(r"D:\Epic Games\AmongUs") },
        ];
        let running = [PathBuf::from(r"C:\Windows\explorer.exe"), PathBuf::from(r"d:\epic games\amongus\Among Us.exe")];
        assert_eq!(running_game(&installed, &running).as_deref(), Some("epic:Among"));
        assert_eq!(running_game(&installed, &[PathBuf::from(r"D:\Epic Games\rocketleague 2\game.exe")]), None);
    }
}
