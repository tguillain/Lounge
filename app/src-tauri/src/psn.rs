//! PlayStation Network friends, through the API the PlayStation App uses.
//! Sony doesn't document it: the sign-in follows the community psn-api library.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::friends::{self, Friend, FriendsResult, Network, Presence};

const AUTH_URL: &str = "https://ca.account.sony.com/api/authz/v3/oauth";
const USERS_URL: &str = "https://m.np.playstation.com/api/userProfile/v1/internal/users";
const CLIENT_ID: &str = "09515159-7237-4370-9b40-3806e67c0891";
const CLIENT_AUTHORIZATION: &str = "Basic MDk1MTUxNTktNzIzNy00MzcwLTliNDAtMzgwNmU2N2MwODkxOnVjUGprYTV0bnRCMktxc1A=";
const REDIRECT_URI: &str = "com.scee.psxandroid.scecompcall://redirect";
const SCOPE: &str = "psn:mobile.v2.core psn:clientapp";
pub const SIGN_IN_PAGE: &str = "https://www.playstation.com/";
pub const TOKEN_PAGE: &str = "https://ca.account.sony.com/api/v1/ssocookie";
const KEYRING_SERVICE: &str = "Lounge";
const KEYRING_USER: &str = "psn-refresh-token";
const BATCH: usize = 100;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const EXPIRY_MARGIN: Duration = Duration::from_secs(60);
const EXPIRED: &str = "Your PlayStation sign-in has expired. Connect again from Settings.";

struct AccessToken {
    value: String,
    valid_until: Instant,
}

#[derive(Clone)]
struct Profile {
    name: String,
    avatar: Option<String>,
}

static ACCESS: Mutex<Option<AccessToken>> = Mutex::new(None);
static PROFILES: LazyLock<Mutex<HashMap<String, Profile>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

enum Failure {
    Unauthorized,
    Message(String),
}

impl From<Failure> for String {
    fn from(failure: Failure) -> Self {
        match failure {
            Failure::Unauthorized => EXPIRED.into(),
            Failure::Message(message) => message,
        }
    }
}

pub fn load() -> FriendsResult {
    if stored_refresh_token().is_none() {
        return FriendsResult::NotConfigured;
    }
    match fetch_friends() {
        Ok(friends) => FriendsResult::Ready { friends },
        Err(message) => FriendsResult::Failed { message },
    }
}

/// Signs in with the NPSSO token the user copied from Sony's page, keeps
/// only the long-lived refresh token, and checks the friends list answers.
pub fn connect(pasted: &str) -> Result<(), String> {
    let npsso = npsso_from(pasted).ok_or("Paste the npsso value from Sony's page: 64 letters and digits.")?;
    let code = exchange_npsso(&npsso)?;
    let tokens = request_tokens(&[
        ("code", code.as_str()),
        ("redirect_uri", REDIRECT_URI),
        ("grant_type", "authorization_code"),
        ("token_format", "jwt"),
    ])
    .map_err(|_| "Sony refused this token. Copy a fresh one and try again.".to_string())?;
    let refresh = tokens.refresh.clone().ok_or("Sony didn't keep you signed in. Try again.")?;
    remember_access(&tokens);
    friend_ids().map_err(String::from)?;
    keyring_entry()?.set_password(&refresh).map_err(|error| error.to_string())
}

pub fn disconnect() -> Result<(), String> {
    *lock(&ACCESS) = None;
    lock(&PROFILES).clear();
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn npsso_from(pasted: &str) -> Option<String> {
    let pasted = pasted.trim();
    let value = match pasted.find("\"npsso\"") {
        Some(start) => pasted[start + 7..].trim_start_matches([' ', ':', '"']).split('"').next()?,
        None => pasted.trim_matches('"'),
    };
    (value.len() == 64 && value.chars().all(|c| c.is_ascii_alphanumeric())).then(|| value.to_string())
}

fn fetch_friends() -> Result<Vec<Friend>, String> {
    let ids = friend_ids()?;
    let profiles = profiles(&ids)?;
    let mut presences = HashMap::new();
    for batch in ids.chunks(BATCH) {
        presences.extend(presence_batch(batch)?);
    }
    let mut friends: Vec<Friend> = ids
        .iter()
        .filter_map(|id| {
            let profile = profiles.get(id)?;
            Some(to_friend(id, profile, presences.get(id)))
        })
        .collect();
    friends::sort(&mut friends);
    Ok(friends)
}

fn friend_ids() -> Result<Vec<String>, Failure> {
    let answer = get(&format!("{USERS_URL}/me/friends?limit=1000"))?;
    Ok(strings(answer.get("friends")))
}

fn profiles(ids: &[String]) -> Result<HashMap<String, Profile>, String> {
    let missing: Vec<String> = {
        let known = lock(&PROFILES);
        ids.iter().filter(|id| !known.contains_key(*id)).cloned().collect()
    };
    for batch in missing.chunks(BATCH) {
        let answer = get(&format!("{USERS_URL}/profiles?accountIds={}", batch.join(",")))?;
        let found = list(&answer, "profiles");
        let mut known = lock(&PROFILES);
        for (index, entry) in found.iter().enumerate() {
            let id = entry.get("accountId").and_then(Value::as_str).map(str::to_string).or_else(|| batch.get(index).cloned());
            if let (Some(id), Some(profile)) = (id, parse_profile(entry)) {
                known.insert(id, profile);
            }
        }
    }
    let known = lock(&PROFILES);
    Ok(ids.iter().filter_map(|id| known.get(id).map(|profile| (id.clone(), profile.clone()))).collect())
}

fn presence_batch(ids: &[String]) -> Result<HashMap<String, Value>, Failure> {
    let answer = get(&format!("{USERS_URL}/basicPresences?type=primary&accountIds={}", ids.join(",")))?;
    let found = list(&answer, "basicPresences");
    Ok(found
        .into_iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let id = entry.get("accountId").and_then(Value::as_str).map(str::to_string).or_else(|| ids.get(index).cloned())?;
            Some((id, entry.get("basicPresence").cloned().unwrap_or(entry)))
        })
        .collect())
}

fn list(answer: &Value, field: &str) -> Vec<Value> {
    answer.get(field).or(Some(answer)).and_then(Value::as_array).cloned().unwrap_or_default()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn parse_profile(entry: &Value) -> Option<Profile> {
    let name = entry.get("onlineId")?.as_str()?.to_string();
    let avatars = entry.get("avatars").and_then(Value::as_array);
    let sized = |wanted: &str| avatars?.iter().find(|avatar| avatar.get("size").and_then(Value::as_str) == Some(wanted));
    let avatar = sized("m")
        .or_else(|| sized("l"))
        .or_else(|| avatars?.last())
        .and_then(|avatar| avatar.get("url")?.as_str())
        .map(str::to_string);
    Some(Profile { name, avatar })
}

fn to_friend(id: &str, profile: &Profile, presence: Option<&Value>) -> Friend {
    let platform_info = presence.and_then(|presence| presence.get("primaryPlatformInfo"));
    let text = |value: Option<&Value>, field: &str| value.and_then(|value| value.get(field)).and_then(Value::as_str).map(str::to_string);
    let online_status = text(platform_info, "onlineStatus").or_else(|| text(presence, "onlineStatus"));
    let platform = text(platform_info, "platform").or_else(|| text(presence, "platform")).map(|platform| platform.to_uppercase());
    let game = presence
        .and_then(|presence| presence.get("gameTitleInfoList"))
        .and_then(Value::as_array)
        .and_then(|games| games.first())
        .and_then(|game| game.get("titleName")?.as_str())
        .map(str::to_string);
    let online = online_status.as_deref() == Some("online");
    let available = text(presence, "availability").as_deref() != Some("unavailable");
    let presence = match (online, game.is_some(), available) {
        (false, _, _) => Presence::Offline,
        (true, true, _) => Presence::InGame,
        (true, false, true) => Presence::Online,
        (true, false, false) => Presence::Away,
    };
    Friend {
        id: format!("psn:{id}"),
        network: Network::Psn,
        name: profile.name.clone(),
        avatar: profile.avatar.clone(),
        presence,
        game: if online { game } else { None },
        platform: if online { platform } else { None },
        join_uri: None,
    }
}

fn get(url: &str) -> Result<Value, Failure> {
    match get_once(url, &access_token()?) {
        Err(Failure::Unauthorized) => {
            *lock(&ACCESS) = None;
            get_once(url, &access_token()?)
        }
        other => other,
    }
}

fn get_once(url: &str, token: &str) -> Result<Value, Failure> {
    match ureq::get(url).timeout(REQUEST_TIMEOUT).set("Authorization", &format!("Bearer {token}")).call() {
        Ok(response) => response.into_json().map_err(|_| Failure::Message("PlayStation sent an unexpected answer.".into())),
        Err(ureq::Error::Status(401, _)) => Err(Failure::Unauthorized),
        Err(ureq::Error::Status(403, _)) => Err(Failure::Message("PlayStation won't share this friends list. Check your PlayStation privacy settings.".into())),
        Err(ureq::Error::Status(code, _)) => Err(Failure::Message(format!("PlayStation is not answering right now (error {code})."))),
        Err(ureq::Error::Transport(_)) => Err(Failure::Message("Couldn't reach PlayStation. Check your internet connection.".into())),
    }
}

fn access_token() -> Result<String, Failure> {
    if let Some(token) = lock(&ACCESS).as_ref().filter(|token| token.valid_until > Instant::now()) {
        return Ok(token.value.clone());
    }
    let refresh = stored_refresh_token().ok_or(Failure::Unauthorized)?;
    let tokens = request_tokens(&[
        ("refresh_token", refresh.as_str()),
        ("grant_type", "refresh_token"),
        ("token_format", "jwt"),
        ("scope", SCOPE),
    ])?;
    if let Some(renewed) = tokens.refresh.as_deref().filter(|renewed| *renewed != refresh)
        && let Ok(entry) = keyring_entry()
    {
        let _ = entry.set_password(renewed);
    }
    remember_access(&tokens);
    Ok(tokens.access)
}

struct Tokens {
    access: String,
    expires_in: u64,
    refresh: Option<String>,
}

fn request_tokens(form: &[(&str, &str)]) -> Result<Tokens, Failure> {
    let response = ureq::post(&format!("{AUTH_URL}/token"))
        .timeout(REQUEST_TIMEOUT)
        .set("Authorization", CLIENT_AUTHORIZATION)
        .send_form(form);
    let answer: Value = match response {
        Ok(response) => response.into_json().map_err(|_| Failure::Message("PlayStation sent an unexpected answer.".into()))?,
        Err(ureq::Error::Status(400 | 401, _)) => return Err(Failure::Unauthorized),
        Err(ureq::Error::Status(code, _)) => return Err(Failure::Message(format!("PlayStation sign-in is not answering (error {code})."))),
        Err(ureq::Error::Transport(_)) => return Err(Failure::Message("Couldn't reach PlayStation. Check your internet connection.".into())),
    };
    let access = answer.get("access_token").and_then(Value::as_str).ok_or(Failure::Unauthorized)?.to_string();
    Ok(Tokens {
        access,
        expires_in: answer.get("expires_in").and_then(Value::as_u64).unwrap_or(3600),
        refresh: answer.get("refresh_token").and_then(Value::as_str).map(str::to_string),
    })
}

fn remember_access(tokens: &Tokens) {
    let lifetime = Duration::from_secs(tokens.expires_in).saturating_sub(EXPIRY_MARGIN);
    *lock(&ACCESS) = Some(AccessToken { value: tokens.access.clone(), valid_until: Instant::now() + lifetime });
}

fn exchange_npsso(npsso: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().redirects(0).timeout(REQUEST_TIMEOUT).build();
    let query = [
        ("access_type", "offline"),
        ("client_id", CLIENT_ID),
        ("redirect_uri", REDIRECT_URI),
        ("response_type", "code"),
        ("scope", SCOPE),
    ];
    let mut request = agent.get(&format!("{AUTH_URL}/authorize")).set("Cookie", &format!("npsso={npsso}"));
    for (name, value) in query {
        request = request.query(name, value);
    }
    let response = request.call().map_err(|error| match error {
        ureq::Error::Transport(_) => "Couldn't reach PlayStation. Check your internet connection.".to_string(),
        ureq::Error::Status(code, _) => format!("PlayStation sign-in refused the token (error {code})."),
    })?;
    response
        .header("location")
        .and_then(code_from_redirect)
        .ok_or("This token didn't work. Sign in to playstation.com again and copy a fresh one.".into())
}

pub fn code_from_redirect(location: &str) -> Option<String> {
    let query = location.split_once('?')?.1;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("code="))
        .filter(|code| !code.is_empty())
        .map(str::to_string)
}

fn stored_refresh_token() -> Option<String> {
    keyring_entry().ok()?.get_password().ok()
}

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|error| error.to_string())
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TOKEN: &str = "AbCdEfGhIjKlMnOpQrStUvWxYz0123456789AbCdEfGhIjKlMnOpQrStUvWxYz01";

    #[test]
    fn accepts_the_token_alone_or_the_whole_answer_from_sony() {
        assert_eq!(npsso_from(TOKEN).as_deref(), Some(TOKEN));
        assert_eq!(npsso_from(&format!("  \"{TOKEN}\"\n")).as_deref(), Some(TOKEN));
        assert_eq!(npsso_from(&format!("{{\"npsso\":\"{TOKEN}\"}}")).as_deref(), Some(TOKEN));
        assert_eq!(npsso_from(&format!("{{ \"npsso\": \"{TOKEN}\" }}")).as_deref(), Some(TOKEN));
        assert_eq!(npsso_from("too short"), None);
    }

    #[test]
    fn reads_the_code_from_the_sign_in_redirect() {
        let location = "com.scee.psxandroid.scecompcall://redirect/?code=v3.AbC123&cid=4f1d";
        assert_eq!(code_from_redirect(location).as_deref(), Some("v3.AbC123"));
        assert_eq!(code_from_redirect("com.scee.psxandroid.scecompcall://redirect/?error=login_required"), None);
    }

    #[test]
    fn picks_a_mid_sized_avatar() {
        let profile = parse_profile(&json!({
            "onlineId": "Kaori_93",
            "avatars": [
                { "size": "s", "url": "https://psn/small.png" },
                { "size": "m", "url": "https://psn/medium.png" },
                { "size": "l", "url": "https://psn/large.png" }
            ]
        }))
        .unwrap();
        assert_eq!(profile.name, "Kaori_93");
        assert_eq!(profile.avatar.as_deref(), Some("https://psn/medium.png"));
    }

    #[test]
    fn turns_a_presence_into_a_friend() {
        let profile = Profile { name: "Kaori_93".into(), avatar: None };
        let playing = json!({
            "availability": "availableToPlay",
            "primaryPlatformInfo": { "onlineStatus": "online", "platform": "ps5" },
            "gameTitleInfoList": [{ "titleName": "Astro Bot", "format": "PS5" }]
        });
        let friend = to_friend("123", &profile, Some(&playing));
        assert_eq!(friend.id, "psn:123");
        assert_eq!(friend.network, Network::Psn);
        assert_eq!(friend.presence, Presence::InGame);
        assert_eq!(friend.game.as_deref(), Some("Astro Bot"));
        assert_eq!(friend.platform.as_deref(), Some("PS5"));

        let online = json!({ "availability": "availableToPlay", "primaryPlatformInfo": { "onlineStatus": "online", "platform": "PS4" } });
        assert_eq!(to_friend("1", &profile, Some(&online)).presence, Presence::Online);

        let offline = json!({ "availability": "unavailable", "primaryPlatformInfo": { "onlineStatus": "offline", "platform": "PS5" }, "gameTitleInfoList": [{ "titleName": "Astro Bot" }] });
        let friend = to_friend("2", &profile, Some(&offline));
        assert_eq!(friend.presence, Presence::Offline);
        assert_eq!(friend.game, None);
        assert_eq!(to_friend("3", &profile, None).presence, Presence::Offline);
    }

    #[test]
    fn reads_batch_answers_wrapped_or_bare() {
        let wrapped = json!({ "profiles": [{ "onlineId": "A" }] });
        assert_eq!(list(&wrapped, "profiles").len(), 1);
        let bare = json!([{ "onlineId": "A" }, { "onlineId": "B" }]);
        assert_eq!(list(&bare, "profiles").len(), 2);
    }

    #[test]
    #[ignore = "calls PlayStation with the sign-in stored on this PC"]
    fn reads_this_account() {
        match load() {
            FriendsResult::Ready { friends } => {
                let online = friends.iter().filter(|friend| friend.presence != Presence::Offline).count();
                let playing = friends.iter().filter(|friend| friend.game.is_some()).count();
                let avatars = friends.iter().filter(|friend| friend.avatar.is_some()).count();
                println!("friends: {}, online: {online}, playing: {playing}, with avatar: {avatars}", friends.len());
            }
            FriendsResult::NotConfigured => println!("not configured"),
            FriendsResult::Failed { message } => println!("failed: {message}"),
        }
    }
}
