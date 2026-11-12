mod achievements;
mod autostart;
mod dualsense_output;
mod epic;
mod focus;
mod friends;
mod game;
mod lightbar;
mod logo;
mod pad;
mod quick_menu;
mod session;
mod settings;
mod steam;
mod update;
mod vdf;
mod watcher;
mod window;

use std::path::PathBuf;

use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent};
use tauri_plugin_opener::OpenerExt;

use game::Game;
use session::SessionState;
use window::WindowState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    app_id: u32,
    started_at: u64,
    game: Option<Game>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Preferences {
    start_with_windows: bool,
    start_with_windows_available: bool,
    start_fullscreen: bool,
    vibration: bool,
    show_friends: bool,
    sounds: bool,
    lightbar: bool,
    version: String,
    can_update: bool,
}

const ALLOWED_LAUNCH_PREFIXES: [&str; 3] = ["steam://rungameid/", "steam://joinlobby/", "com.epicgames.launcher://apps/"];
const STEAM_API_KEY_PAGE: &str = "https://steamcommunity.com/dev/apikey";
const RELEASES_PAGE: &str = "https://github.com/tguillain/lounge/releases";
const ONE_INSTANCE_ONLY: bool = !cfg!(debug_assertions);

#[tauri::command]
async fn get_friends() -> Result<friends::FriendsResult, String> {
    tauri::async_runtime::spawn_blocking(friends::load).await.map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_achievements(app_id: u32) -> Result<achievements::AchievementsResult, String> {
    tauri::async_runtime::spawn_blocking(move || achievements::load(app_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn save_steam_api_key(key: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || friends::save_key(&key))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn forget_steam_api_key() -> Result<(), String> {
    friends::forget_key()
}

#[tauri::command]
fn open_steam_api_key_page(app: tauri::AppHandle) -> Result<(), String> {
    app.opener().open_url(STEAM_API_KEY_PAGE, None::<&str>).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_releases_page(app: tauri::AppHandle) -> Result<(), String> {
    app.opener().open_url(RELEASES_PAGE, None::<&str>).map_err(|error| error.to_string())
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<Option<update::AvailableUpdate>, String> {
    update::check(&app).await
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    update::install(&app).await
}

#[tauri::command]
fn get_controller_battery(pad: tauri::State<'_, pad::PadControl>) -> Option<pad::Battery> {
    pad.battery()
}

#[tauri::command]
async fn set_lightbar(app: tauri::AppHandle, image: Option<String>, fallback_hue: f32) -> Result<(), String> {
    let colors_app = app.clone();
    let color = tauri::async_runtime::spawn_blocking(move || {
        image
            .filter(|path| !path.starts_with("http"))
            .and_then(|path| colors_app.state::<lightbar::ColorCache>().color_of(&path))
    })
    .await
    .map_err(|error| error.to_string())?
    .unwrap_or_else(|| lightbar::hue_color(fallback_hue));
    app.state::<pad::PadControl>().set_light(Some(color));
    Ok(())
}

#[tauri::command]
fn release_lightbar(pad: tauri::State<'_, pad::PadControl>) {
    pad.set_light(None);
}

#[tauri::command]
fn pulse_controller(pad: tauri::State<'_, pad::PadControl>) {
    pad.pulse();
}

fn logo_cache_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_cache_dir().ok()?.join("logos");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

#[tauri::command]
fn get_preferences(app: tauri::AppHandle) -> Result<Preferences, String> {
    let saved = settings::load(&settings::path(&app)?);
    Ok(Preferences {
        start_with_windows: autostart::is_enabled(),
        start_with_windows_available: autostart::AVAILABLE,
        start_fullscreen: saved.start_fullscreen,
        vibration: saved.vibration,
        show_friends: saved.show_friends,
        sounds: saved.sounds,
        lightbar: saved.lightbar,
        version: app.package_info().version.to_string(),
        can_update: update::AVAILABLE,
    })
}

#[tauri::command]
fn set_start_with_windows(enabled: bool) -> Result<(), String> {
    autostart::set_enabled(enabled)
}

#[tauri::command]
fn set_toggle(app: tauri::AppHandle, toggle: settings::Toggle, enabled: bool) -> Result<(), String> {
    settings::set_toggle(&settings::path(&app)?, toggle, enabled)
}

#[tauri::command]
async fn get_library(app: tauri::AppHandle) -> Result<Vec<Game>, String> {
    let logo_cache = logo_cache_dir(&app);
    let hidden = settings::load(&settings::path(&app)?).hidden_games;
    tauri::async_runtime::spawn_blocking(move || {
        let mut games: Vec<Game> = steam::scan(logo_cache.as_deref()).into_iter().chain(epic::scan()).collect();
        for game in &mut games {
            game.hidden = hidden.contains(&game.id);
        }
        games.sort_by(|a, b| b.last_played.cmp(&a.last_played).then_with(|| a.name.cmp(&b.name)));
        games
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_game_hidden(app: tauri::AppHandle, game_id: String, hidden: bool) -> Result<(), String> {
    settings::set_hidden(&settings::path(&app)?, &game_id, hidden)
}

#[tauri::command]
async fn get_session(app: tauri::AppHandle) -> Result<Option<SessionInfo>, String> {
    let Some(current) = session::current(&app) else {
        return Ok(None);
    };
    let logo_cache = logo_cache_dir(&app);
    let game = tauri::async_runtime::spawn_blocking(move || steam::find_game(current.app_id, logo_cache.as_deref()))
        .await
        .map_err(|error| error.to_string())?;
    Ok(Some(SessionInfo { app_id: current.app_id, started_at: current.started_at_ms, game }))
}

async fn focus_running_game(app: &tauri::AppHandle) {
    if let Some(current) = session::current(app) {
        let _ = tauri::async_runtime::spawn_blocking(move || session::focus_game(current.app_id)).await;
    }
}

#[tauri::command]
async fn quick_menu_resume(app: tauri::AppHandle) {
    focus_running_game(&app).await;
    quick_menu::close(&app);
}

#[tauri::command]
async fn back_to_game(app: tauri::AppHandle) -> Result<(), String> {
    if session::current(&app).is_none() {
        return Err("No game is running".into());
    }
    focus_running_game(&app).await;
    window::close_for_game(&app);
    Ok(())
}

#[tauri::command]
async fn quick_menu_open_lounge(app: tauri::AppHandle) {
    quick_menu::close(&app);
    window::open_main(&app);
}

#[tauri::command]
async fn quick_menu_quit_game(app: tauri::AppHandle) -> Result<(), String> {
    let current = session::current(&app).ok_or("No game is running")?;
    tauri::async_runtime::spawn_blocking(move || session::kill_game(current.app_id))
        .await
        .map_err(|error| error.to_string())??;
    quick_menu::close(&app);
    Ok(())
}

#[tauri::command]
fn launch_game(app: tauri::AppHandle, uri: String) -> Result<(), String> {
    if !ALLOWED_LAUNCH_PREFIXES.iter().any(|prefix| uri.starts_with(prefix)) {
        return Err(format!("Refused launch link: {uri}"));
    }
    app.opener().open_url(uri, None::<&str>).map_err(|error| error.to_string())
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Lounge", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Lounge")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => window::open_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::open_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    if ONE_INSTANCE_ONLY {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::open_main(app)));
    }
    let app = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(update::UpdateState::default())
        .manage(lightbar::ColorCache::default())
        .manage(WindowState::default())
        .manage(SessionState::default())
        .setup(|app| {
            let scope = app.asset_protocol_scope();
            if let Some(root) = steam::steam_root() {
                let _ = scope.allow_directory(steam::image_cache_dir(&root), true);
            }
            if let Some(logos) = logo_cache_dir(app.handle()) {
                let _ = scope.allow_directory(logos, false);
            }
            window::build_main(app.handle())?;
            build_tray(app)?;
            watcher::spawn(app.handle().clone());
            pad::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            set_game_hidden,
            get_preferences,
            set_start_with_windows,
            set_toggle,
            open_releases_page,
            check_for_update,
            install_update,
            pulse_controller,
            get_controller_battery,
            set_lightbar,
            release_lightbar,
            launch_game,
            get_session,
            get_friends,
            get_achievements,
            save_steam_api_key,
            forget_steam_api_key,
            open_steam_api_key_page,
            back_to_game,
            quick_menu_resume,
            quick_menu_open_lounge,
            quick_menu_quit_game
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { code: None, api, .. } = event {
            if app.state::<WindowState>().closed_for_game() {
                api.prevent_exit();
            }
        }
    });
}
