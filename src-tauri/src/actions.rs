//! What happens when a hotkey is pressed.

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager};

use crate::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Devanagari ↔ Latin, replacing the selection.
    Toggle,
    /// Keep the selection and add the transliteration next to it.
    Alongside,
}

/// Run an action on a worker thread (never on the UI thread); ignore
/// presses while a previous one is still running.
pub fn trigger(app: &AppHandle, action: Action) {
    let state = app.state::<AppState>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        run(&app, action);
        app.state::<AppState>().busy.store(false, Ordering::SeqCst);
    });
}

fn compute(app: &AppHandle, action: Action, text: &str) -> Option<String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let mut session = state.session.lock().unwrap();
    match action {
        Action::Toggle => session.toggle(text, &settings.transliteration),
        Action::Alongside => {
            session.alongside(text, &settings.alongside_template, &settings.transliteration)
        }
    }
}

#[cfg(windows)]
fn run(app: &AppHandle, action: Action) {
    use std::thread::sleep;
    use std::time::Duration;

    use unicode_segmentation::UnicodeSegmentation;

    use crate::settings::OutputMethod;
    use crate::win::{beep, clipboard, input, uia, ComGuard};

    let _com = ComGuard::new();
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();

    input::wait_for_modifier_release(Duration::from_millis(1500));

    // 1. Read the selection: UI Automation first, clipboard as fallback.
    let mut saved: Option<clipboard::Snapshot> = None;
    let mut text = if settings.read_selection_via_ui_automation { uia::selected_text() } else { None };
    if text.is_some() {
        log::info!("{action:?}: selection read via UI Automation");
    } else {
        saved = Some(clipboard::snapshot());
        text = clipboard::copy_selection(Duration::from_millis(600));
        log::info!("{action:?}: selection read via clipboard ({})", if text.is_some() { "ok" } else { "nothing copied" });
    }
    let restore = |saved: Option<clipboard::Snapshot>| {
        if settings.restore_clipboard {
            if let Some(s) = saved {
                s.restore();
            }
        }
    };
    let Some(text) = text else {
        log::info!("no selection");
        restore(saved);
        beep();
        return;
    };

    // 2. Transliterate.
    let Some(output) = compute(app, action, &text) else {
        restore(saved);
        if action == Action::Toggle {
            beep();
        }
        return;
    };

    // 3. Replace the selection.
    match settings.output_method {
        OutputMethod::Paste => {
            if saved.is_none() {
                saved = Some(clipboard::snapshot());
            }
            if !clipboard::set_text(&output) {
                restore(saved);
                beep();
                return;
            }
            input::send_paste();
            sleep(Duration::from_millis(150));
        }
        OutputMethod::Type => {
            input::type_text(&output);
            sleep(Duration::from_millis(60));
        }
    }

    // 4. Re-select the inserted text so the toggle hotkey can be pressed again.
    // UI Automation first; otherwise Shift+Left, but only for Latin output:
    // caret steps are one per grapheme there, while each app moves through
    // Devanagari clusters differently.
    if settings.reselect_after_replace && action == Action::Toggle && !uia::select_before_caret(&output) {
        let latin = !output.chars().any(sanscribe_core::is_devanagari);
        let n = output.replace("\r\n", "\n").graphemes(true).count();
        if latin && n <= 2000 {
            input::shift_left(n);
        }
    }

    // 5. Put the user's clipboard back once the target app has read ours.
    if saved.is_some() {
        sleep(Duration::from_millis(350));
        restore(saved);
    }
}

#[cfg(not(windows))]
fn run(app: &AppHandle, action: Action) {
    // The OS integration is Windows-only for now (macOS will follow).
    log::warn!("{action:?} is not implemented on this platform");
    let _ = compute(app, action, "");
}
