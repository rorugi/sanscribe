use std::path::PathBuf;

use sanscribe_core::Options;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMethod {
    /// Paste via the clipboard (fast; the user's clipboard is restored afterwards).
    #[default]
    Paste,
    /// Type the characters as Unicode key events (never touches the clipboard,
    /// but Word's AutoCorrect may change the text).
    Type,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub transliteration: Options,
    pub hotkey_toggle: String,
    pub hotkey_alongside: String,
    pub alongside_template: String,
    /// AltGr dead keys for typing diacritics (ā ṛ ṅ ś ñ …).
    pub diacritic_keys: bool,
    pub read_selection_via_ui_automation: bool,
    pub output_method: OutputMethod,
    pub reselect_after_replace: bool,
    pub restore_clipboard: bool,
    pub launch_at_login: bool,
    pub check_for_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            transliteration: Options::default(),
            hotkey_toggle: "Ctrl+Shift+Alt+T".into(),
            hotkey_alongside: "Ctrl+Shift+Alt+A".into(),
            alongside_template: "{original} ({result})".into(),
            diacritic_keys: false,
            read_selection_via_ui_automation: true,
            output_method: OutputMethod::Paste,
            reselect_after_replace: true,
            restore_clipboard: true,
            launch_at_login: true,
            check_for_updates: true,
        }
    }
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

/// Returns the settings and whether a settings file existed (false on first run).
pub fn load(app: &AppHandle) -> (Settings, bool) {
    let Some(p) = path(app) else { return (Settings::default(), false) };
    match std::fs::read_to_string(&p) {
        Ok(s) => (serde_json::from_str(&s).unwrap_or_default(), true),
        Err(_) => (Settings::default(), false),
    }
}

pub fn save(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let p = path(app).ok_or("no config directory")?;
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(&p, json).map_err(|e| e.to_string())
}
