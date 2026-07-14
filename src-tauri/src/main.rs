// Prevents an extra console window on Windows in release; irrelevant on
// macOS but harmless to keep for cross-platform Tauri convention.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod onboarding;
mod settings;
mod snapshot;
mod tray;

use settings::Settings;
use snapshot::UsageSnapshot;
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

const SETTINGS_STORE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

struct AppState {
    settings: Mutex<Settings>,
}

fn load_settings(app: &AppHandle) -> Settings {
    app.store(SETTINGS_STORE)
        .ok()
        .and_then(|store| store.get(SETTINGS_KEY))
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

fn save_settings(app: &AppHandle, settings: &Settings) {
    if let Ok(store) = app.store(SETTINGS_STORE) {
        if let Ok(value) = serde_json::to_value(settings) {
            store.set(SETTINGS_KEY, value);
            let _ = store.save();
        }
    }
}

#[tauri::command]
fn get_snapshots() -> Vec<UsageSnapshot> {
    snapshot::read_all_snapshots()
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_settings(app: AppHandle, state: State<AppState>, settings: Settings) {
    let autostart = app.autolaunch();
    let _ = if settings.launch_at_login {
        autostart.enable()
    } else {
        autostart.disable()
    };
    *state.settings.lock().unwrap() = settings.clone();
    save_settings(&app, &settings);
    refresh_tray(&app, &state);
}

/// Lets the popover force an immediate re-read of the snapshot directory
/// and tray update, instead of waiting for the next poll loop tick.
#[tauri::command]
fn refresh_now(app: AppHandle, state: State<AppState>) {
    refresh_tray(&app, &state);
}

fn refresh_tray(app: &AppHandle, state: &State<AppState>) {
    let snapshots = snapshot::read_all_snapshots();
    snapshot::mirror_to_app_group(&snapshots);
    let settings = state.settings.lock().unwrap();
    let title = tray::tray_title(&snapshots, &settings);
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(title));
    }
    // Lets the popover re-render immediately when a refresh is triggered
    // from the tray menu or poll loop, not just from its own button.
    let _ = app.emit("usage-refreshed", ());
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            refresh_tray(app, &app.state::<AppState>());
        }
    }
}

fn start_poll_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        let state = app.state::<AppState>();
        let minutes = state.settings.lock().unwrap().poll_interval_minutes.max(1);
        refresh_tray(&app, &state);
        std::thread::sleep(Duration::from_secs(minutes as u64 * 60));
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            settings: Mutex::new(Settings::default()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let loaded = load_settings(&handle);
            let autostart = handle.autolaunch();
            let _ = if loaded.launch_at_login {
                autostart.enable()
            } else {
                autostart.disable()
            };
            *handle.state::<AppState>().settings.lock().unwrap() = loaded;

            let show_item = MenuItem::with_id(app, "show", "Open", true, None::<&str>)?;
            let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&show_item, &refresh_item, &separator, &quit_item],
            )?;

            let tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .title("–")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => toggle_main_window(app),
                    "refresh" => refresh_tray(app, &app.state::<AppState>()),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Tauri emits a Click event for both the mouse-down and
                    // mouse-up of a single click; reacting to both toggles
                    // the window twice (show then immediately hide again),
                    // which looks like the popover flashing and vanishing.
                    // Only the button-up transition is a completed click.
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            app.manage(tray);

            // Clicking the window's close button should hide it, not quit
            // the app — the tray icon is the app's only durable UI, and
            // losing it would strand the user with no way to reopen the
            // window short of relaunching.
            if let Some(window) = app.get_webview_window("main") {
                let window_for_handler = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_for_handler.hide();
                    }
                });
            }

            start_poll_loop(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshots,
            get_settings,
            update_settings,
            refresh_now,
            onboarding::check_statusline,
            onboarding::install_statusline
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Belt-and-suspenders: the window's own CloseRequested handler
            // already prevents a real close, but ignore ExitRequested too so
            // hiding the window can never be misread as "quit the app."
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
