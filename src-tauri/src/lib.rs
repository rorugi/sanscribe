mod actions;
mod settings;
mod tray;
mod updater;
#[cfg(windows)]
mod win;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use sanscribe_core::Session;
use tauri::{AppHandle, Manager, RunEvent, State, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use actions::Action;
use settings::Settings;

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub session: Mutex<Session>,
    pub busy: AtomicBool,
    /// Registered hotkey id → action.
    pub hotkeys: Mutex<HashMap<u32, Action>>,
}

fn parse_hotkey(s: &str) -> Result<Shortcut, String> {
    s.parse::<Shortcut>().map_err(|e| format!("Invalid shortcut \"{s}\": {e}"))
}

/// (Re-)register both global hotkeys from the settings.
fn register_hotkeys(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let toggle = parse_hotkey(&s.hotkey_toggle)?;
    let alongside = parse_hotkey(&s.hotkey_alongside)?;
    if toggle.id() == alongside.id() {
        return Err("The two hotkeys must be different.".into());
    }
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for (sc, action, text) in [(toggle, Action::Toggle, &s.hotkey_toggle), (alongside, Action::Alongside, &s.hotkey_alongside)] {
        gs.register(sc).map_err(|e| {
            format!("{text} could not be registered (probably used by another application): {e}")
        })?;
        map.insert(sc.id(), action);
    }
    *app.state::<AppState>().hotkeys.lock().unwrap() = map;
    tray::update_labels(app, s);
    Ok(())
}

fn apply_autostart(app: &AppHandle, enabled: bool) {
    let al = app.autolaunch();
    let current = al.is_enabled().unwrap_or(false);
    let res = match (enabled, current) {
        (true, false) => al.enable(),
        (false, true) => al.disable(),
        _ => Ok(()),
    };
    if let Err(e) = res {
        log::warn!("autostart: {e}");
    }
}

pub fn show_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

// ---------- commands for the settings window ----------

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn default_settings() -> Settings {
    Settings::default()
}

#[tauri::command]
fn save_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<(), String> {
    let previous = state.settings.lock().unwrap().clone();
    if let Err(e) = register_hotkeys(&app, &settings) {
        // Keep the old, working hotkeys.
        let _ = register_hotkeys(&app, &previous);
        return Err(e);
    }
    apply_autostart(&app, settings.launch_at_login);
    settings::save(&app, &settings)?;
    *state.settings.lock().unwrap() = settings;
    // Options may have changed; cached conversions would no longer match.
    state.session.lock().unwrap().forget();
    Ok(())
}

/// Live preview in the settings window (uses a fresh session).
#[tauri::command]
fn preview(text: String, alongside: bool, settings: Settings) -> Option<String> {
    let mut s = Session::new();
    if alongside {
        s.alongside(&text, &settings.alongside_template, &settings.transliteration)
    } else {
        s.toggle(&text, &settings.transliteration)
    }
}

#[tauri::command]
fn check_for_updates(app: AppHandle) {
    updater::check(app, true);
}

#[tauri::command]
fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

// ---------- entry point ----------

pub fn run() {
    let app = tauri::Builder::default()
        // Must be the first plugin.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_settings(app)))
        .plugin(
            // Logs to the console (dev) and %LOCALAPPDATA%\com.sanscribe.app\logs.
            tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build(),
        )
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let action = app.state::<AppState>().hotkeys.lock().unwrap().get(&shortcut.id()).copied();
                    if let Some(action) = action {
                        actions::trigger(app, action);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let (settings, existed) = settings::load(&handle);
            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
                session: Mutex::new(Session::new()),
                busy: AtomicBool::new(false),
                hotkeys: Mutex::new(HashMap::new()),
            });
            tray::create(&handle)?;
            if let Err(e) = register_hotkeys(&handle, &settings) {
                log::error!("{e}");
                show_settings(&handle);
            }
            if !existed {
                // First run: save defaults, enable autostart, show the settings once.
                apply_autostart(&handle, settings.launch_at_login);
                let _ = settings::save(&handle, &settings);
                show_settings(&handle);
            }
            if settings.check_for_updates {
                updater::check(handle.clone(), false);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the settings window only hides it; the app lives in the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            default_settings,
            save_settings,
            preview,
            check_for_updates,
            app_version
        ])
        .build(tauri::generate_context!())
        .expect("error while building Sanscribe");

    app.run(|_app, event| {
        // Keep running in the tray when no window is visible.
        if let RunEvent::ExitRequested { api, code: None, .. } = event {
            api.prevent_exit();
        }
    });
}
