//! Synthetic keyboard input via SendInput.

use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_C, VK_CONTROL,
    VK_LEFT, VK_LWIN, VK_MENU, VK_RETURN, VK_RWIN, VK_SHIFT, VK_V,
};

const MODIFIERS: [VIRTUAL_KEY; 5] = [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN];

fn key(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    }
}

fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    // Large batches are split so a slow target app doesn't drop events.
    for chunk in inputs.chunks(200) {
        unsafe {
            SendInput(chunk, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

fn is_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
}

/// The hotkey fires while the user still holds its modifiers. Sending Ctrl+C
/// while Alt is held would arrive as Ctrl+Alt+C, so wait for the release
/// (and release them ourselves if the user keeps holding them).
pub fn wait_for_modifier_release(timeout: Duration) {
    let start = Instant::now();
    while MODIFIERS.iter().any(|vk| is_down(*vk)) {
        if start.elapsed() > timeout {
            let ups: Vec<INPUT> = MODIFIERS
                .iter()
                .filter(|vk| is_down(**vk))
                .map(|vk| key(*vk, 0, KEYEVENTF_KEYUP))
                .collect();
            send(&ups);
            break;
        }
        sleep(Duration::from_millis(10));
    }
}

fn ctrl_plus(vk: VIRTUAL_KEY) {
    send(&[
        key(VK_CONTROL, 0, KEYBD_EVENT_FLAGS(0)),
        key(vk, 0, KEYBD_EVENT_FLAGS(0)),
        key(vk, 0, KEYEVENTF_KEYUP),
        key(VK_CONTROL, 0, KEYEVENTF_KEYUP),
    ]);
}

pub fn send_copy() {
    ctrl_plus(VK_C);
}

pub fn send_paste() {
    ctrl_plus(VK_V);
}

/// Extend the selection to the left by `n` caret positions.
pub fn shift_left(n: usize) {
    let mut v = Vec::with_capacity(n * 2 + 2);
    v.push(key(VK_SHIFT, 0, KEYBD_EVENT_FLAGS(0)));
    for _ in 0..n {
        v.push(key(VK_LEFT, 0, KEYEVENTF_EXTENDEDKEY));
        v.push(key(VK_LEFT, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP));
    }
    v.push(key(VK_SHIFT, 0, KEYEVENTF_KEYUP));
    send(&v);
}

/// Type text directly as Unicode key events (no clipboard involved).
/// Line breaks are sent as Enter.
pub fn type_text(text: &str) {
    let mut v = Vec::with_capacity(text.len() * 2);
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    for c in normalized.chars() {
        if c == '\n' {
            v.push(key(VK_RETURN, 0, KEYBD_EVENT_FLAGS(0)));
            v.push(key(VK_RETURN, 0, KEYEVENTF_KEYUP));
            continue;
        }
        let mut buf = [0u16; 2];
        for unit in c.encode_utf16(&mut buf) {
            v.push(key(VIRTUAL_KEY(0), *unit, KEYEVENTF_UNICODE));
            v.push(key(VIRTUAL_KEY(0), *unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
        }
    }
    send(&v);
}
