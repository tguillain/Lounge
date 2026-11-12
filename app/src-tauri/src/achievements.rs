use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{friends, steam};

const API_BASE: &str = "https://api.steampowered.com/ISteamUserStats";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const DEFAULT_LANGUAGE: &str = "english";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub unlocked_at: Option<u64>,
    pub hidden: bool,
    pub rarity: Option<f32>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum AchievementsResult {
    None,
    Ready { unlocked: u32, total: u32, achievements: Vec<Achievement>, complete: bool },
    Failed { message: String },
}

enum WebError {
    NoAchievements,
    Private,
    Other(String),
}

#[derive(Deserialize)]
struct PlayerResponse {
    playerstats: PlayerStats,
}

#[derive(Deserialize)]
struct PlayerStats {
    #[serde(default)]
    achievements: Vec<PlayerAchievement>,
}

#[derive(Deserialize)]
struct PlayerAchievement {
    apiname: String,
    achieved: u8,
    #[serde(default)]
    unlocktime: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct SchemaResponse {
    game: SchemaGame,
}

#[derive(Deserialize, Default)]
struct SchemaGame {
    #[serde(default, rename = "availableGameStats")]
    stats: Option<SchemaStats>,
}

#[derive(Deserialize, Default)]
struct SchemaStats {
    #[serde(default)]
    achievements: Vec<SchemaAchievement>,
}

#[derive(Deserialize)]
struct SchemaAchievement {
    name: String,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    hidden: u8,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Deserialize)]
struct LocalAchievement {
    #[serde(rename = "strID")]
    id: String,
    #[serde(rename = "strName")]
    name: String,
    #[serde(default, rename = "strDescription")]
    description: Option<String>,
    #[serde(default, rename = "strImage")]
    image: Option<String>,
    #[serde(default, rename = "bAchieved")]
    achieved: bool,
    #[serde(default, rename = "rtUnlocked")]
    unlocked_at: u64,
    #[serde(default, rename = "bHidden")]
    hidden: bool,
    #[serde(default, rename = "flAchieved")]
    rarity: f32,
}

#[derive(Deserialize)]
struct LocalSummary {
    #[serde(default, rename = "vecHighlight")]
    highlight: Vec<LocalAchievement>,
    #[serde(default, rename = "vecAchievedHidden")]
    other_unlocked: Vec<LocalAchievement>,
    #[serde(default, rename = "vecUnachieved")]
    locked: Vec<LocalAchievement>,
    #[serde(default, rename = "nTotal")]
    total: u32,
    #[serde(default, rename = "nAchieved")]
    unlocked: u32,
}

pub fn load(app_id: u32) -> AchievementsResult {
    let steam_id = friends::current_steam_id();
    let local = || steam_id.as_deref().and_then(|id| local_cache(app_id, id));
    let (Some(key), Some(id)) = (friends::stored_key(), steam_id.as_deref()) else {
        return local().map_or(AchievementsResult::None, |content| from_local_cache(&content));
    };
    match fetch(&key, id, app_id) {
        Ok(result) => result,
        Err(WebError::NoAchievements) => AchievementsResult::None,
        Err(error) => match local() {
            Some(content) => from_local_cache(&content),
            None => AchievementsResult::Failed {
                message: match error {
                    WebError::Private => "Your Steam game details are private. Set them to public in your Steam privacy settings.".into(),
                    WebError::Other(message) => message,
                    WebError::NoAchievements => unreachable!(),
                },
            },
        },
    }
}

fn fetch(key: &str, steam_id: &str, app_id: u32) -> Result<AchievementsResult, WebError> {
    let language = steam::language().unwrap_or_else(|| DEFAULT_LANGUAGE.into());
    let player_url = format!("{API_BASE}/GetPlayerAchievements/v1/?key={key}&steamid={steam_id}&appid={app_id}&l={language}");
    let schema_url = format!("{API_BASE}/GetSchemaForGame/v2/?key={key}&appid={app_id}&l={language}");
    let rarity_url = format!("{API_BASE}/GetGlobalAchievementPercentagesForApp/v2/?gameid={app_id}");
    let (player, schema, rarity) = std::thread::scope(|scope| {
        let schema = scope.spawn(|| get_json::<SchemaResponse>(&schema_url).ok());
        let rarity = scope.spawn(|| get_json::<Value>(&rarity_url).ok());
        let player = get_json::<PlayerResponse>(&player_url);
        (player, schema.join().ok().flatten(), rarity.join().ok().flatten())
    });
    let player = player?;
    let schema = schema.and_then(|response| response.game.stats).map(|stats| stats.achievements).unwrap_or_default();
    let rarity = rarity.map(|value| parse_rarity(&value)).unwrap_or_default();
    Ok(merge(player.playerstats.achievements, schema, &rarity))
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, WebError> {
    match ureq::get(url).timeout(REQUEST_TIMEOUT).call() {
        Ok(response) => response.into_json().map_err(|_| WebError::Other("Steam sent an unexpected answer.".into())),
        Err(ureq::Error::Status(400, _)) => Err(WebError::NoAchievements),
        Err(ureq::Error::Status(401 | 403, _)) => Err(WebError::Private),
        Err(ureq::Error::Status(code, _)) => Err(WebError::Other(format!("Steam is not answering right now (error {code})."))),
        Err(ureq::Error::Transport(_)) => Err(WebError::Other("Couldn't reach Steam. Check your internet connection.".into())),
    }
}

fn parse_rarity(value: &Value) -> HashMap<String, f32> {
    let list = value.pointer("/achievementpercentages/achievements").and_then(Value::as_array);
    list.into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?.to_string();
            let percent = entry.get("percent")?;
            let percent = percent.as_f64().or_else(|| percent.as_str()?.parse().ok())?;
            Some((name, percent as f32))
        })
        .collect()
}

fn merge(player: Vec<PlayerAchievement>, schema: Vec<SchemaAchievement>, rarity: &HashMap<String, f32>) -> AchievementsResult {
    if player.is_empty() {
        return AchievementsResult::None;
    }
    let schema: HashMap<String, SchemaAchievement> = schema.into_iter().map(|entry| (entry.name.clone(), entry)).collect();
    let achievements: Vec<Achievement> = player
        .into_iter()
        .map(|entry| {
            let details = schema.get(&entry.apiname);
            let name = entry
                .name
                .filter(|name| !name.is_empty())
                .or_else(|| details.and_then(|d| d.display_name.clone()))
                .unwrap_or_else(|| entry.apiname.clone());
            let description = entry
                .description
                .or_else(|| details.and_then(|d| d.description.clone()))
                .filter(|text| !text.trim().is_empty());
            Achievement {
                rarity: rarity.get(&entry.apiname).copied(),
                id: entry.apiname,
                name,
                description,
                icon: details.and_then(|d| d.icon.clone()),
                unlocked_at: (entry.achieved == 1).then_some(entry.unlocktime),
                hidden: details.is_some_and(|d| d.hidden == 1),
            }
        })
        .collect();
    let unlocked = achievements.iter().filter(|achievement| achievement.unlocked_at.is_some()).count() as u32;
    AchievementsResult::Ready { unlocked, total: achievements.len() as u32, achievements, complete: true }
}

fn local_cache(app_id: u32, steam_id: &str) -> Option<String> {
    let account_id = steam_id.parse::<u64>().ok()?.checked_sub(friends::STEAM_ID64_BASE)?;
    let path: PathBuf = steam::steam_root()?
        .join("userdata")
        .join(account_id.to_string())
        .join("config")
        .join("librarycache")
        .join(format!("{app_id}.json"));
    fs::read_to_string(path).ok()
}

pub fn from_local_cache(content: &str) -> AchievementsResult {
    let Some(summary) = local_summary(content) else {
        return AchievementsResult::None;
    };
    if summary.total == 0 {
        return AchievementsResult::None;
    }
    let mut seen = HashSet::new();
    let achievements: Vec<Achievement> = summary
        .highlight
        .into_iter()
        .chain(summary.other_unlocked)
        .chain(summary.locked)
        .filter(|entry| seen.insert(entry.id.clone()))
        .map(|entry| Achievement {
            id: entry.id,
            name: entry.name,
            description: entry.description.filter(|text| !text.trim().is_empty()),
            icon: entry.image,
            unlocked_at: entry.achieved.then_some(entry.unlocked_at),
            hidden: entry.hidden,
            rarity: (entry.rarity > 0.0).then_some(entry.rarity),
        })
        .collect();
    let complete = achievements.len() as u32 >= summary.total;
    AchievementsResult::Ready { unlocked: summary.unlocked, total: summary.total, achievements, complete }
}

fn local_summary(content: &str) -> Option<LocalSummary> {
    let entries: Vec<(String, Value)> = serde_json::from_str(content).ok()?;
    let (_, section) = entries.into_iter().find(|(name, _)| name == "achievements")?;
    serde_json::from_value(section.get("data")?.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: &str, achieved: u8, unlocktime: u64) -> PlayerAchievement {
        PlayerAchievement { apiname: id.into(), achieved, unlocktime, name: None, description: None }
    }

    fn schema(id: &str, name: &str, hidden: u8) -> SchemaAchievement {
        SchemaAchievement {
            name: id.into(),
            display_name: Some(name.into()),
            description: Some(format!("Do {name}")),
            hidden,
            icon: Some(format!("https://cdn/{id}.jpg")),
        }
    }

    #[test]
    fn merges_progress_with_names_icons_and_rarity() {
        let rarity = HashMap::from([("WIN".to_string(), 12.5)]);
        let result = merge(
            vec![player("WIN", 1, 1_700_000_000), player("SECRET", 0, 0)],
            vec![schema("WIN", "Winner", 0), schema("SECRET", "Secret", 1)],
            &rarity,
        );
        let AchievementsResult::Ready { unlocked, total, achievements, complete } = result else {
            panic!("expected achievements");
        };
        assert_eq!((unlocked, total, complete), (1, 2, true));
        assert_eq!(achievements[0].name, "Winner");
        assert_eq!(achievements[0].unlocked_at, Some(1_700_000_000));
        assert_eq!(achievements[0].rarity, Some(12.5));
        assert_eq!(achievements[0].icon.as_deref(), Some("https://cdn/WIN.jpg"));
        assert!(achievements[1].hidden);
        assert_eq!(achievements[1].unlocked_at, None);
    }

    #[test]
    fn treats_a_game_without_achievements_as_none() {
        assert_eq!(merge(vec![], vec![], &HashMap::new()), AchievementsResult::None);
    }

    #[test]
    fn reads_rarity_written_as_numbers_or_text() {
        let value = serde_json::json!({ "achievementpercentages": { "achievements": [
            { "name": "A", "percent": 40.5 },
            { "name": "B", "percent": "3.2" }
        ] } });
        let rarity = parse_rarity(&value);
        assert_eq!(rarity.get("A"), Some(&40.5));
        assert_eq!(rarity.get("B"), Some(&3.2));
    }

    #[test]
    fn reads_the_steam_library_cache() {
        let content = r#"[["achievements",{"version":2,"data":{
            "vecHighlight":[{"strID":"1","strName":"First","strDescription":"Start","strImage":"https://cdn/1.jpg","bAchieved":true,"rtUnlocked":1700000000,"flAchieved":63.3}],
            "vecAchievedHidden":[{"strID":"2","strName":"Second","bAchieved":true,"rtUnlocked":1690000000,"flAchieved":0}],
            "vecUnachieved":[{"strID":"3","strName":"Third","strDescription":"","bAchieved":false,"rtUnlocked":0,"bHidden":true,"flAchieved":2.2}],
            "nTotal":5,"nAchieved":2}}],["descriptions",{"version":1,"data":{}}]]"#;
        let AchievementsResult::Ready { unlocked, total, achievements, complete } = from_local_cache(content) else {
            panic!("expected achievements");
        };
        assert_eq!((unlocked, total, complete), (2, 5, false));
        assert_eq!(achievements.len(), 3);
        assert_eq!(achievements[1].rarity, None);
        assert_eq!(achievements[2].description, None);
        assert!(achievements[2].hidden);
    }

    #[test]
    fn ignores_a_cache_with_no_achievements() {
        let content = r#"[["achievements",{"version":2,"data":{"vecHighlight":[],"vecUnachieved":[],"vecAchievedHidden":[],"nTotal":0,"nAchieved":0}}]]"#;
        assert_eq!(from_local_cache(content), AchievementsResult::None);
    }

    #[test]
    #[ignore = "calls Steam with the key stored on this PC"]
    fn loads_achievements_of_local_games() {
        for app_id in [1_454_400, 3_768_760, 730, 1_172_470] {
            match load(app_id) {
                AchievementsResult::Ready { unlocked, total, achievements, complete } => {
                    let first = achievements.first().map(|a| (a.name.clone(), a.icon.is_some(), a.rarity));
                    println!("{app_id}: {unlocked}/{total} complete={complete} listed={} first={first:?}", achievements.len());
                }
                other => println!("{app_id}: {other:?}"),
            }
        }
    }
}
