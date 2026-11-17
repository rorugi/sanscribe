//! UI Automation: read the selection of the focused control without touching
//! the clipboard, and re-select inserted text so the hotkey can toggle again.
//! Works in apps that implement the UIA TextPattern (Word, Outlook, Notepad,
//! WordPad, most WPF/WinUI apps, Chromium and Firefox in most text fields).
//! Callers fall back to the clipboard when this returns `None`/`false`.

use windows::core::{Interface, BSTR};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationTextPattern,
    IUIAutomationTextRange, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    UIA_TextPatternId,
};

fn automation() -> Option<IUIAutomation> {
    let a: IUIAutomation = unsafe {
        CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
            .or_else(|_| CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER))
    }
    .ok()?;
    // Never hang on an unresponsive target app.
    if let Ok(a2) = a.cast::<IUIAutomation2>() {
        unsafe {
            let _ = a2.SetConnectionTimeout(800);
            let _ = a2.SetTransactionTimeout(800);
        }
    }
    Some(a)
}

fn focused_text_pattern(a: &IUIAutomation) -> Option<IUIAutomationTextPattern> {
    unsafe {
        let el = a.GetFocusedElement().ok()?;
        el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId).ok()
    }
}

fn single_selection(tp: &IUIAutomationTextPattern) -> Option<IUIAutomationTextRange> {
    unsafe {
        let sel = tp.GetSelection().ok()?;
        if sel.Length().ok()? != 1 {
            return None;
        }
        sel.GetElement(0).ok()
    }
}

/// Text currently selected in the focused control, if the app exposes it.
/// Must be called on a thread with COM initialised.
pub fn selected_text() -> Option<String> {
    let a = automation()?;
    let tp = focused_text_pattern(&a)?;
    let range = single_selection(&tp)?;
    let text = unsafe { range.GetText(-1) }.ok()?.to_string();
    (!text.trim().is_empty()).then_some(text)
}

/// Select `text` if it ends exactly at the caret (i.e. we just inserted it).
pub fn select_before_caret(text: &str) -> bool {
    let Some(a) = automation() else { return false };
    let Some(tp) = focused_text_pattern(&a) else { return false };
    let Some(caret) = single_selection(&tp) else { return false };
    unsafe {
        let Ok(doc) = tp.DocumentRange() else { return false };
        let Ok(before) = doc.Clone() else { return false };
        if before
            .MoveEndpointByRange(TextPatternRangeEndpoint_End, &caret, TextPatternRangeEndpoint_Start)
            .is_err()
        {
            return false;
        }
        // Editors store line breaks differently (Word uses \r).
        let lf = text.replace("\r\n", "\n");
        let candidates = [lf.clone(), lf.replace('\n', "\r"), lf.replace('\n', "\r\n")];
        for c in candidates.iter() {
            let Ok(found) = before.FindText(&BSTR::from(c.as_str()), true, false) else { continue };
            let ends_at_caret = found
                .CompareEndpoints(TextPatternRangeEndpoint_End, &caret, TextPatternRangeEndpoint_Start)
                .map(|d| d == 0)
                .unwrap_or(false);
            if ends_at_caret && found.Select().is_ok() {
                return true;
            }
        }
    }
    false
}
