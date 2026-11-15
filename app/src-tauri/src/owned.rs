//! The Steam games you own but haven't installed, from the Steam Web API.
//! Steam keeps only installed games on the PC, so this needs the same key
//! as the friends list.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::friends;
use crate::game::{Game, GameImages, Store};
use crate::steam;

const OWNED_GAMES_URL: &str = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/";
const STORE_ITEMS_URL: &str = "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/";
const STORE_ASSETS_BASE: &str = "https://shared.cloudflare.steamstatic.com/store_item_assets/";
const STORE_BATCH: usize = 40;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum OwnedResult {
    NotConfigured,
    Ready { games: Vec<Game> },
    Failed { message: String },
}

#[derive(Debug, Deserialize)]
struct OwnedResponse {
    response: OwnedGames,
}

#[derive(Debug, Default, Deserialize)]
struct OwnedGames {
    #[serde(default)]
    games: Vec<OwnedGame>,
}

#[derive(Debug, Deserialize)]
pub struct OwnedGame {
    appid: u32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    playtime_forever: u64,
    #[serde(default)]
    playtime_2weeks: u64,
    #[serde(default)]
    rtime_last_played: u64,
}

#[derive(Debug, Deserialize)]
struct ItemsResponse {
    response: Items,
}

#[derive(Debug, Default, Deserialize)]
struct Items {
    #[serde(default)]
    store_items: Vec<StoreItem>,
}

#[derive(Debug, Deserialize)]
struct StoreItem {
    #[serde(default)]
    appid: u32,
    #[serde(default)]
    assets: Option<StoreAssets>,
}

/// Where the store keeps a game's artwork today: recent games live under a
/// hashed folder, so the old /steam/apps/<id>/<file> addresses fail for them.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct StoreAssets {
    #[serde(default)]
    asset_url_format: Option<String>,
    #[serde(default)]
    library_capsule: Option<String>,
    #[serde(default)]
    library_hero: Option<String>,
    #[serde(default)]
    header: Option<String>,
}

impl StoreAssets {
    fn url(&self, file: Option<&String>) -> Option<String> {
        let file = file.filter(|file| !file.is_empty())?;
        let format = self.asset_url_format.as_deref()?;
        Some(format!("{STORE_ASSETS_BASE}{}", format.replace("${FILENAME}", file)))
    }
}

/// Every owned game, installed or not: the interface drops the ones it
/// already has from the PC, which also know their size and local artwork.
pub fn load(hidden: &BTreeSet<String>) -> OwnedResult {
    let Some(key) = friends::stored_key() else {
        return OwnedResult::NotConfigured;
    };
    let Some(steam_id) = friends::current_steam_id() else {
        return OwnedResult::Failed { message: "Couldn't find which Steam account is signed in on this PC.".into() };
    };
    match fetch(&key, &steam_id) {
        Ok(owned) => {
            let cache = steam::steam_root().map(|root| steam::image_cache_dir(&root));
            let missing: Vec<u32> = owned
                .iter()
                .filter(|game| cache.as_deref().is_none_or(|cache| local_cover(cache, game.appid).is_none()))
                .map(|game| game.appid)
                .collect();
            let store = store_assets(&missing);
            OwnedResult::Ready { games: to_games(owned, hidden, cache.as_deref(), &store) }
        }
        Err(message) => OwnedResult::Failed { message },
    }
}

fn fetch(key: &str, steam_id: &str) -> Result<Vec<OwnedGame>, String> {
    let response = ureq::get(OWNED_GAMES_URL)
        .timeout(REQUEST_TIMEOUT)
        .query("key", key)
        .query("steamid", steam_id)
        .query("include_appinfo", "1")
        .query("include_played_free_games", "1")
        .call();
    match response {
        Ok(response) => response
            .into_json::<OwnedResponse>()
            .map(|answer| answer.response.games)
            .map_err(|_| "Steam sent an unexpected answer.".to_string()),
        Err(ureq::Error::Status(401 | 403, _)) => Err("Steam refused to list your games. Check your Steam Web API key.".into()),
        Err(ureq::Error::Status(code, _)) => Err(format!("Steam is not answering right now (error {code}).")),
        Err(ureq::Error::Transport(_)) => Err("Couldn't reach Steam. Check your internet connection.".into()),
    }
}

fn store_assets(app_ids: &[u32]) -> HashMap<u32, StoreAssets> {
    let mut found = HashMap::new();
    for batch in app_ids.chunks(STORE_BATCH) {
        let ids: Vec<String> = batch.iter().map(|id| format!("{{\"appid\":{id}}}")).collect();
        let input = format!(
            "{{\"ids\":[{}],\"context\":{{\"language\":\"english\",\"country_code\":\"US\"}},\"data_request\":{{\"include_assets\":true}}}}",
            ids.join(",")
        );
        let answer = ureq::get(STORE_ITEMS_URL).timeout(REQUEST_TIMEOUT).query("input_json", &input).call();
        let Ok(items) = answer.map_err(|_| ()).and_then(|response| response.into_json::<ItemsResponse>().map_err(|_| ())) else {
            continue;
        };
        for item in items.response.store_items {
            if let Some(assets) = item.assets {
                found.insert(item.appid, assets);
            }
        }
    }
    found
}

fn local_cover(cache: &Path, app_id: u32) -> Option<String> {
    let id = app_id.to_string();
    ["library_capsule.jpg", "library_600x900.jpg"].iter().find_map(|file| steam::find_cached_image(cache, &id, file))
}

/// Steam's own cache first, then the store's current addresses, then the
/// old CDN ones, which still work for older games.
pub fn images_for(app_id: u32, cache: Option<&Path>, store: Option<&StoreAssets>) -> GameImages {
    let id = app_id.to_string();
    let local = |files: &[&str]| cache.and_then(|cache| files.iter().find_map(|file| steam::find_cached_image(cache, &id, file)));
    let remote = |pick: fn(&StoreAssets) -> Option<&String>| store.and_then(|assets| assets.url(pick(assets)));
    GameImages {
        hero: local(&["library_hero.jpg"])
            .or_else(|| remote(|assets| assets.library_hero.as_ref()))
            .or_else(|| Some(steam::cdn_image(app_id, "library_hero.jpg"))),
        logo: local(&["logo.png"]).or_else(|| Some(steam::cdn_image(app_id, "logo.png"))),
        logo_is_dark: false,
        header: local(&["library_header.jpg", "header.jpg"])
            .or_else(|| remote(|assets| assets.header.as_ref()))
            .or_else(|| Some(steam::cdn_image(app_id, "header.jpg"))),
        cover: cache
            .and_then(|cache| local_cover(cache, app_id))
            .or_else(|| remote(|assets| assets.library_capsule.as_ref()))
            .or_else(|| Some(steam::cdn_image(app_id, "library_600x900.jpg"))),
    }
}

pub fn to_games(owned: Vec<OwnedGame>, hidden: &BTreeSet<String>, cache: Option<&Path>, store: &HashMap<u32, StoreAssets>) -> Vec<Game> {
    owned
        .into_iter()
        .filter(|game| !game.name.trim().is_empty() && !steam::is_excluded(&game.appid.to_string()))
        .map(|game| {
            let id = format!("steam:{}", game.appid);
            Game {
                hidden: hidden.contains(&id),
                id,
                store: Store::Steam,
                name: game.name,
                last_played: game.rtime_last_played,
                size_on_disk: 0,
                play_time: Some(game.playtime_forever),
                recent_play_time: Some(game.playtime_2weeks),
                launch_uri: format!("steam://rungameid/{}", game.appid),
                images: images_for(game.appid, cache, store.get(&game.appid)),
                installed: false,
                download: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_owned_games_answer() {
        let answer: OwnedResponse = serde_json::from_str(
            r#"{ "response": { "game_count": 2, "games": [
                { "appid": 220, "name": "Half-Life 2", "playtime_forever": 845, "rtime_last_played": 1600000000, "img_icon_url": "x" },
                { "appid": 570, "name": "Dota 2", "playtime_forever": 12, "playtime_2weeks": 12 }
            ] } }"#,
        )
        .unwrap();
        let games = to_games(answer.response.games, &BTreeSet::from(["steam:570".to_string()]), None, &HashMap::new());
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].id, "steam:220");
        assert_eq!(games[0].play_time, Some(845));
        assert_eq!(games[0].recent_play_time, Some(0));
        assert!(!games[0].installed);
        assert!(games[0].images.cover.as_deref().unwrap().ends_with("/220/library_600x900.jpg"));
        assert!(games[1].hidden);
    }

    #[test]
    fn copes_with_a_private_or_empty_library() {
        let answer: OwnedResponse = serde_json::from_str(r#"{ "response": {} }"#).unwrap();
        assert!(answer.response.games.is_empty());
    }

    #[test]
    fn leaves_out_tools_and_nameless_entries() {
        let owned = vec![
            OwnedGame { appid: 228980, name: "Steamworks Common Redistributables".into(), playtime_forever: 0, playtime_2weeks: 0, rtime_last_played: 0 },
            OwnedGame { appid: 10, name: " ".into(), playtime_forever: 0, playtime_2weeks: 0, rtime_last_played: 0 },
        ];
        assert!(to_games(owned, &BTreeSet::new(), None, &HashMap::new()).is_empty());
    }

    #[test]
    fn prefers_the_local_cache_then_the_store_addresses() {
        let cache = std::env::temp_dir().join(format!("lounge-owned-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&cache);
        std::fs::create_dir_all(cache.join("2807960").join("3059e94f")).unwrap();
        std::fs::write(cache.join("2807960").join("3059e94f").join("library_capsule.jpg"), b"jpg").unwrap();
        let store = StoreAssets {
            asset_url_format: Some("steam/apps/2807960/${FILENAME}?t=1794335892".into()),
            library_capsule: Some("5efba891/library_capsule.jpg".into()),
            library_hero: Some("7165682c/library_hero.jpg".into()),
            header: None,
        };
        let images = images_for(2807960, Some(&cache), Some(&store));
        assert!(images.cover.unwrap().ends_with("library_capsule.jpg"));
        assert_eq!(
            images.hero.as_deref(),
            Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2807960/7165682c/library_hero.jpg?t=1794335892")
        );
        assert!(images.header.unwrap().ends_with("/2807960/header.jpg"));
        let remote_only = images_for(2807960, None, Some(&store));
        assert!(remote_only.cover.unwrap().contains("/5efba891/library_capsule.jpg"));
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn reads_the_store_items_answer() {
        let answer: ItemsResponse = serde_json::from_str(
            r#"{ "response": { "store_items": [
                { "appid": 462930, "assets": { "asset_url_format": "steam/apps/462930/${FILENAME}?t=1", "library_capsule": "portrait.png", "header": "header.jpg" } },
                { "appid": 10, "success": 2 }
            ] } }"#,
        )
        .unwrap();
        assert_eq!(answer.response.store_items.len(), 2);
        assert!(answer.response.store_items[1].assets.is_none());
    }

    #[test]
    #[ignore = "calls Steam with the key stored on this PC"]
    fn reads_this_account() {
        match load(&BTreeSet::new()) {
            OwnedResult::Ready { games } => {
                let played = games.iter().filter(|game| game.play_time.unwrap_or(0) > 0).count();
                let covers: Vec<&str> = games.iter().filter_map(|game| game.images.cover.as_deref()).collect();
                let local = covers.iter().filter(|cover| !cover.starts_with("http")).count();
                let store = covers.iter().filter(|cover| cover.contains("store_item_assets")).count();
                println!("owned: {}, played: {played}", games.len());
                println!("covers: {local} from Steam's cache, {store} from the store, {} from the old CDN", covers.len() - local - store);
                if let Some(battlefield) = games.iter().find(|game| game.id == "steam:2807960") {
                    println!("Battlefield 6 cover: {:?}", battlefield.images.cover);
                }
            }
            OwnedResult::NotConfigured => println!("no key"),
            OwnedResult::Failed { message } => println!("failed: {message}"),
        }
    }
}
