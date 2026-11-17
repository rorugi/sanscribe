use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::settings::Settings;

/// Menu items that show the current hotkeys.
struct HotkeyLabels(MenuItem<Wry>, MenuItem<Wry>);
struct Labels(Mutex<Option<HotkeyLabels>>);

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "info-toggle", "", false, None::<&str>)?;
    let alongside = MenuItem::with_id(app, "info-alongside", "", false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let update = MenuItem::with_id(app, "update", "Check for updates", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Sanscribe", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &toggle,
            &alongside,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &update,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    app.manage(Labels(Mutex::new(Some(HotkeyLabels(toggle, alongside)))));

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Sanscribe – Devanagari ↔ IAST")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => crate::show_settings(app),
            "update" => crate::updater::check(app.clone(), true),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                crate::show_settings(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn update_labels(app: &AppHandle, s: &Settings) {
    if let Some(labels) = app.try_state::<Labels>() {
        if let Some(HotkeyLabels(t, a)) = labels.0.lock().unwrap().as_ref() {
            let _ = t.set_text(format!("Toggle script:  {}", s.hotkey_toggle));
            let _ = a.set_text(format!("Insert alongside:  {}", s.hotkey_alongside));
        }
    }
}
