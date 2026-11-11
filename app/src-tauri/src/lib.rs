mod epic;
mod focus;
mod game;
mod logo;
mod pad;
mod quick_menu;
mod session;
mod settings;
mod steam;
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

const ALLOWED_LAUNCH_PREFIXES: [&str; 2] = ["steam://rungameid/", "com.epicgames.launcher://apps/"];

fn logo_cache_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_cache_dir().ok()?.join("logos");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_library(app: tauri::AppHandle) -> Result<Vec<Game>, String> {
    let logo_cache = logo_cache_dir(&app);
    let hidden = settings::load(&settings_path(&app)?).hidden_games;
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
    settings::set_hidden(&settings_path(&app)?, &game_id, hidden)
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
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            build_tray(app)?;
            watcher::spawn(app.handle().clone());
            pad::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            set_game_hidden,
            launch_game,
            get_session,
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
