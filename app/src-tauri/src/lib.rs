mod epic;
mod game;
mod logo;
mod settings;
mod steam;
mod vdf;
mod watcher;
mod window;

use std::path::PathBuf;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent};
use tauri_plugin_opener::OpenerExt;

use game::Game;
use window::WindowState;

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
fn launch_game(app: tauri::AppHandle, uri: String) -> Result<(), String> {
    if !ALLOWED_LAUNCH_PREFIXES.iter().any(|prefix| uri.starts_with(prefix)) {
        return Err(format!("Lien de lancement refusé : {uri}"));
    }
    app.opener().open_url(uri, None::<&str>).map_err(|error| error.to_string())
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Ouvrir le launcher", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Game Launcher")
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_library, set_game_hidden, launch_game])
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
