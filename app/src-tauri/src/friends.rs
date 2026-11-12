use std::cmp::Reverse;
use std::fs;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{steam, vdf};

const API_BASE: &str = "https://api.steampowered.com/ISteamUser";
const KEYRING_SERVICE: &str = "Lounge";
const KEYRING_USER: &str = "steam-web-api-key";
pub const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;
const SUMMARIES_BATCH: usize = 100;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const STATE_OFFLINE: u8 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Presence {
    InGame,
    Online,
    Away,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub steam_id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub presence: Presence,
    pub game: Option<String>,
    pub join_uri: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum FriendsResult {
    NotConfigured,
    Ready { friends: Vec<Friend> },
    Failed { message: String },
}

#[derive(Debug, Deserialize)]
pub struct Player {
    steamid: String,
    personaname: String,
    #[serde(default)]
    avatarmedium: Option<String>,
    #[serde(default)]
    personastate: u8,
    #[serde(default)]
    gameid: Option<String>,
    #[serde(default)]
    gameextrainfo: Option<String>,
    #[serde(default)]
    lobbysteamid: Option<String>,
}

#[derive(Deserialize)]
struct FriendListResponse {
    friendslist: FriendList,
}

#[derive(Deserialize)]
struct FriendList {
    friends: Vec<FriendEntry>,
}

#[derive(Deserialize)]
struct FriendEntry {
    steamid: String,
}

#[derive(Deserialize)]
struct SummariesResponse {
    response: Summaries,
}

#[derive(Deserialize)]
struct Summaries {
    players: Vec<Player>,
}

pub fn load() -> FriendsResult {
    let Some(key) = stored_key() else {
        return FriendsResult::NotConfigured;
    };
    match fetch(&key) {
        Ok(friends) => FriendsResult::Ready { friends },
        Err(message) => FriendsResult::Failed { message },
    }
}

pub fn save_key(key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.len() != 32 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("A Steam Web API key is 32 letters and digits (0-9, A-F).".into());
    }
    fetch(key)?;
    keyring_entry()?.set_password(key).map_err(|error| error.to_string())
}

pub fn forget_key() -> Result<(), String> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn stored_key() -> Option<String> {
    keyring_entry().ok()?.get_password().ok()
}

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|error| error.to_string())
}

fn fetch(key: &str) -> Result<Vec<Friend>, String> {
    let steam_id = current_steam_id().ok_or("Couldn't find which Steam account is signed in on this PC.")?;
    let list: FriendListResponse = get_json(&format!("{API_BASE}/GetFriendList/v1/?key={key}&steamid={steam_id}&relationship=friend"))?;
    let ids: Vec<String> = list.friendslist.friends.into_iter().map(|entry| entry.steamid).collect();
    let mut players = Vec::with_capacity(ids.len());
    for batch in ids.chunks(SUMMARIES_BATCH) {
        let summaries: SummariesResponse = get_json(&format!("{API_BASE}/GetPlayerSummaries/v2/?key={key}&steamids={}", batch.join(",")))?;
        players.extend(summaries.response.players);
    }
    Ok(online_friends(players))
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    match ureq::get(url).timeout(REQUEST_TIMEOUT).call() {
        Ok(response) => response.into_json().map_err(|_| "Steam sent an unexpected answer.".to_string()),
        Err(ureq::Error::Status(401, _)) => Err("Your Steam friends list is private. Set it to public in your Steam privacy settings.".into()),
        Err(ureq::Error::Status(403, _)) => Err("Steam refused this API key. Check it and try again.".into()),
        Err(ureq::Error::Status(code, _)) => Err(format!("Steam is not answering right now (error {code}).")),
        Err(ureq::Error::Transport(_)) => Err("Couldn't reach Steam. Check your internet connection.".into()),
    }
}

pub fn online_friends(players: Vec<Player>) -> Vec<Friend> {
    let mut friends: Vec<Friend> = players
        .into_iter()
        .filter(|player| player.personastate != STATE_OFFLINE || player.gameid.is_some())
        .map(to_friend)
        .collect();
    friends.sort_by_key(|friend| (friend.presence, Reverse(friend.join_uri.is_some()), friend.name.to_lowercase()));
    friends
}

fn to_friend(player: Player) -> Friend {
    let presence = match (player.gameid.is_some(), player.personastate) {
        (true, _) => Presence::InGame,
        (false, 1 | 5 | 6) => Presence::Online,
        _ => Presence::Away,
    };
    let join_uri = match (&player.gameid, &player.lobbysteamid) {
        (Some(game_id), Some(lobby)) => Some(format!("steam://joinlobby/{game_id}/{lobby}/{}", player.steamid)),
        _ => None,
    };
    Friend {
        steam_id: player.steamid,
        name: player.personaname,
        avatar: player.avatarmedium,
        presence,
        game: player.gameextrainfo,
        join_uri,
    }
}

pub fn current_steam_id() -> Option<String> {
    let active = steam::active_account_id().filter(|id| *id != 0);
    if let Some(account_id) = active {
        return Some((STEAM_ID64_BASE + u64::from(account_id)).to_string());
    }
    let root = steam::steam_root()?;
    let content = fs::read_to_string(root.join("config").join("loginusers.vdf")).ok()?;
    most_recent_user(&content)
}

pub fn most_recent_user(content: &str) -> Option<String> {
    let doc = vdf::parse(content)?;
    let users = doc.get("users")?.entries();
    let flagged = users.iter().find(|(_, user)| user.text("MostRecent") == Some("1"));
    let latest = || {
        users
            .iter()
            .max_by_key(|(_, user)| user.text("Timestamp").and_then(|value| value.parse::<u64>().ok()).unwrap_or(0))
    };
    flagged.or_else(latest).map(|(id, _)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(name: &str, state: u8, game: Option<&str>, lobby: Option<&str>) -> Player {
        Player {
            steamid: format!("id-{name}"),
            personaname: name.into(),
            avatarmedium: None,
            personastate: state,
            gameid: game.map(|_| "252950".into()),
            gameextrainfo: game.map(Into::into),
            lobbysteamid: lobby.map(Into::into),
        }
    }

    #[test]
    fn keeps_only_online_friends_and_puts_joinable_games_first() {
        let friends = online_friends(vec![
            player("Zoe", 1, None, None),
            player("Bastien", 0, None, None),
            player("Kaori", 1, Some("Rocket League"), None),
            player("Mathis", 1, Some("Rocket League"), Some("109775")),
            player("Nyx", 3, None, None),
        ]);
        let names: Vec<&str> = friends.iter().map(|friend| friend.name.as_str()).collect();
        assert_eq!(names, ["Mathis", "Kaori", "Zoe", "Nyx"]);
        assert_eq!(friends[0].join_uri.as_deref(), Some("steam://joinlobby/252950/109775/id-Mathis"));
        assert_eq!(friends[3].presence, Presence::Away);
    }

    #[test]
    #[ignore = "reads the Steam account signed in on this PC"]
    fn detects_the_local_steam_account() {
        let id = current_steam_id();
        println!("steam account detected: {}", id.as_ref().is_some_and(|id| id.starts_with("7656119")));
        println!("api key stored: {}", stored_key().is_some());
    }

    #[test]
    fn picks_the_account_flagged_most_recent() {
        let content = r#""users" { "111" { "MostRecent" "0" "Timestamp" "900" } "222" { "MostRecent" "1" "Timestamp" "100" } }"#;
        assert_eq!(most_recent_user(content).as_deref(), Some("222"));
    }

    #[test]
    fn falls_back_to_the_latest_sign_in() {
        let content = r#""users" { "111" { "timestamp" "900" } "222" { "timestamp" "100" } }"#;
        assert_eq!(most_recent_user(content).as_deref(), Some("111"));
    }
}
