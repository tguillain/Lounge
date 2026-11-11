use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::game::{Game, GameImages, Store};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemManifest {
    display_name: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    #[serde(default)]
    install_size: u64,
    #[serde(default)]
    app_categories: Vec<String>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete_install: bool,
}

pub fn scan() -> Vec<Game> {
    let Ok(entries) = fs::read_dir(manifests_dir()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("item")))
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|content| parse_item(&content))
        .collect()
}

pub fn parse_item(content: &str) -> Option<Game> {
    let item: ItemManifest = serde_json::from_str(content).ok()?;
    let is_game = item.app_categories.is_empty() || item.app_categories.iter().any(|c| c == "games");
    if item.incomplete_install || !is_game {
        return None;
    }
    Some(Game {
        id: format!("epic:{}", item.app_name),
        store: Store::Epic,
        launch_uri: format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            item.catalog_namespace, item.catalog_item_id, item.app_name
        ),
        name: item.display_name,
        last_played: 0,
        size_on_disk: item.install_size,
        play_time: None,
        images: GameImages::default(),
        hidden: false,
    })
}

fn manifests_dir() -> PathBuf {
    let program_data = std::env::var_os("PROGRAMDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    program_data.join("Epic").join("EpicGamesLauncher").join("Data").join("Manifests")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROCKET_LEAGUE: &str = r#"{
        "DisplayName": "Rocket League",
        "AppName": "Sugar",
        "CatalogNamespace": "9773aa1aa54f4f7b80e44bef04986cea",
        "CatalogItemId": "530145df28a24424923f5828cc9031a1",
        "InstallSize": 25000000000,
        "AppCategories": ["public", "games", "applications"],
        "bIsIncompleteInstall": false
    }"#;

    #[test]
    fn builds_a_silent_launch_link() {
        let game = parse_item(ROCKET_LEAGUE).unwrap();
        assert_eq!(game.id, "epic:Sugar");
        assert_eq!(
            game.launch_uri,
            "com.epicgames.launcher://apps/9773aa1aa54f4f7b80e44bef04986cea%3A530145df28a24424923f5828cc9031a1%3ASugar?action=launch&silent=true"
        );
    }

    #[test]
    fn skips_incomplete_installs() {
        let partial = ROCKET_LEAGUE.replace("\"bIsIncompleteInstall\": false", "\"bIsIncompleteInstall\": true");
        assert!(parse_item(&partial).is_none());
    }

    #[test]
    fn skips_non_game_entries() {
        let engine = ROCKET_LEAGUE.replace("\"public\", \"games\", \"applications\"", "\"engines\"");
        assert!(parse_item(&engine).is_none());
    }
}
