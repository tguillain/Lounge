use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Serialize;
use serde_json::Value;

use crate::{friends, steam};

const STORE_URL: &str = "https://store.steampowered.com/api/appdetails";
const CACHE_LIFETIME: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_SCREENSHOTS: usize = 12;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Picture {
    pub thumbnail: String,
    pub full: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInfo {
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
    pub release_date: Option<String>,
    pub screenshots: Vec<Picture>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    pub store: Option<StoreInfo>,
    pub install_dir: Option<String>,
    pub screenshots: Vec<Picture>,
}

pub fn load(app_id: u32, cache_dir: Option<&Path>) -> GameDetails {
    GameDetails {
        store: store_info(app_id, cache_dir),
        install_dir: steam::install_path(app_id).map(|path| path.to_string_lossy().into_owned()),
        screenshots: screenshots_dir(app_id).map(|dir| local_screenshots(&dir)).unwrap_or_default(),
    }
}

pub fn screenshots_dir(app_id: u32) -> Option<PathBuf> {
    let account_id = friends::current_steam_id()?.parse::<u64>().ok()?.checked_sub(friends::STEAM_ID64_BASE)?;
    Some(
        steam::steam_root()?
            .join("userdata")
            .join(account_id.to_string())
            .join("760")
            .join("remote")
            .join(app_id.to_string())
            .join("screenshots"),
    )
}

pub fn local_screenshots(dir: &Path) -> Vec<Picture> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            let lower = name.to_lowercase();
            lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png")
        })
        .collect();
    names.sort_unstable_by(|a, b| b.cmp(a));
    names
        .into_iter()
        .take(MAX_SCREENSHOTS)
        .map(|name| {
            let full = dir.join(&name);
            let thumbnail = dir.join("thumbnails").join(&name);
            let thumbnail = if thumbnail.is_file() { thumbnail } else { full.clone() };
            Picture { thumbnail: thumbnail.to_string_lossy().into_owned(), full: full.to_string_lossy().into_owned() }
        })
        .collect()
}

fn store_info(app_id: u32, cache_dir: Option<&Path>) -> Option<StoreInfo> {
    let language = steam::language().unwrap_or_else(|| "english".into());
    let cache = cache_dir.map(|dir| dir.join(format!("{app_id}-{language}.json")));
    if let Some(path) = &cache
        && is_fresh(path)
        && let Ok(content) = fs::read_to_string(path)
    {
        return parse_store(&content, app_id);
    }
    if let Some(content) = fetch(app_id, &language) {
        if let Some(path) = &cache {
            let _ = fs::write(path, &content);
        }
        return parse_store(&content, app_id);
    }
    cache.and_then(|path| fs::read_to_string(path).ok()).and_then(|content| parse_store(&content, app_id))
}

fn is_fresh(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age < CACHE_LIFETIME)
}

fn fetch(app_id: u32, language: &str) -> Option<String> {
    ureq::get(STORE_URL)
        .query("appids", &app_id.to_string())
        .query("l", language)
        .timeout(REQUEST_TIMEOUT)
        .call()
        .ok()?
        .into_string()
        .ok()
        .filter(|content| serde_json::from_str::<Value>(content).is_ok())
}

pub fn parse_store(content: &str, app_id: u32) -> Option<StoreInfo> {
    let root: Value = serde_json::from_str(content).ok()?;
    let entry = root.get(app_id.to_string())?;
    if entry.get("success").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let data = entry.get("data")?;
    let texts = |key: &str, field: Option<&str>| -> Vec<String> {
        data.get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| match field {
                        Some(field) => item.get(field)?.as_str(),
                        None => item.as_str(),
                    })
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };
    let screenshots = data
        .get("screenshots")
        .and_then(Value::as_array)
        .map(|shots| {
            shots
                .iter()
                .filter_map(|shot| {
                    Some(Picture {
                        thumbnail: shot.get("path_thumbnail")?.as_str()?.to_string(),
                        full: shot.get("path_full")?.as_str()?.to_string(),
                    })
                })
                .take(MAX_SCREENSHOTS)
                .collect()
        })
        .unwrap_or_default();
    Some(StoreInfo {
        description: data.get("short_description").and_then(Value::as_str).map(plain_text).filter(|text| !text.is_empty()),
        genres: texts("genres", Some("description")),
        developers: texts("developers", None),
        release_date: data
            .get("release_date")
            .and_then(|date| date.get("date"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|date| !date.is_empty())
            .map(str::to_string),
        screenshots,
    })
}

pub fn plain_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    let decoded = text
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const STORE_SHEET: &str = r#"{"284160":{"success":true,"data":{
        "short_description":"Un simulateur de v&eacute;hicules &quot;soft-body&quot; &amp; plus.<br>Encore.",
        "genres":[{"id":"9","description":"Course automobile"},{"id":"28","description":"Simulation"}],
        "developers":["BeamNG"],
        "release_date":{"coming_soon":false,"date":"29 mai 2015"},
        "screenshots":[{"id":0,"path_thumbnail":"https://cdn/ss_1.600x338.jpg","path_full":"https://cdn/ss_1.1920x1080.jpg"}]
    }}}"#;

    #[test]
    fn reads_the_store_sheet_of_a_game() {
        let info = parse_store(STORE_SHEET, 284160).unwrap();
        assert_eq!(info.genres, ["Course automobile", "Simulation"]);
        assert_eq!(info.developers, ["BeamNG"]);
        assert_eq!(info.release_date.as_deref(), Some("29 mai 2015"));
        assert_eq!(info.screenshots[0].full, "https://cdn/ss_1.1920x1080.jpg");
        assert_eq!(info.description.as_deref(), Some("Un simulateur de v&eacute;hicules \"soft-body\" & plus. Encore."));
    }

    #[test]
    fn has_nothing_for_a_game_without_a_store_page() {
        assert_eq!(parse_store(r#"{"999":{"success":false}}"#, 999), None);
        assert_eq!(parse_store(STORE_SHEET, 1), None);
        assert_eq!(parse_store("not json", 284160), None);
    }

    #[test]
    fn turns_html_into_plain_text_without_decoding_twice() {
        assert_eq!(plain_text("<b>Fast</b> &amp;lt; furious"), "Fast &lt; furious");
        assert_eq!(plain_text("  Line one<br/>line   two "), "Line one line two");
    }

    #[test]
    #[ignore = "reads the real Steam store and this PC's Steam folder"]
    fn reads_a_real_game() {
        let details = load(284160, None);
        println!("{details:#?}");
        assert!(details.store.is_some_and(|store| !store.genres.is_empty()));
    }

    #[test]
    fn lists_the_newest_screenshots_first_with_their_thumbnails() {
        let dir = std::env::temp_dir().join(format!("lounge-shots-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("thumbnails")).unwrap();
        for name in ["20250101120000_1.jpg", "20251231235959_1.jpg", "notes.txt"] {
            fs::write(dir.join(name), b"x").unwrap();
        }
        fs::write(dir.join("thumbnails").join("20251231235959_1.jpg"), b"x").unwrap();
        let shots = local_screenshots(&dir);
        assert_eq!(shots.len(), 2);
        assert!(shots[0].full.ends_with("20251231235959_1.jpg"));
        assert!(shots[0].thumbnail.contains("thumbnails"));
        assert_eq!(shots[1].thumbnail, shots[1].full);
        let _ = fs::remove_dir_all(&dir);
    }
}
