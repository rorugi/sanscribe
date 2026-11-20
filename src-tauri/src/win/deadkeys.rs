//! AltGr dead keys for typing IAST diacritics in any application.
//!
//! Press a mark first, then the letter:
//!   AltGr + .          dot below   ṛ ṭ ḍ ṇ ṣ ṃ ḥ ḷ
//!   AltGr + Shift + .  dot above   ṅ ṁ
//!   AltGr + -          macron      ā ī ū
//!   AltGr + Shift + -  line below  ḻ ṉ ṟ
//!   AltGr + ,          acute       ś
//!   AltGr + Shift + ,  tilde       ñ
//! Marks can be combined (AltGr + . then AltGr + - then r → ṝ).
//! Esc cancels, any other key cancels and is passed through.
//!
//! Implemented with a low-level keyboard hook on its own thread: the mark
//! keys and the letter are swallowed and the composed character is typed
//! as a Unicode key event. Events we inject ourselves are ignored.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

use sanscribe_core::{compose, Mark};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_ESCAPE, VK_LCONTROL,
    VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD,
    VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN, WM_USER,
};

/// Pending marks expire if no letter follows within this time.
const PENDING_TIMEOUT: Duration = Duration::from_secs(5);

struct State {
    marks: Vec<Mark>,
    since: Option<Instant>,
    /// Key-up events to swallow because we swallowed their key-down.
    swallow_up: Vec<u32>,
}

static STATE: Mutex<State> = Mutex::new(State { marks: Vec::new(), since: None, swallow_up: Vec::new() });
/// Thread id of the hook thread (0 = not running).
static THREAD: AtomicU32 = AtomicU32::new(0);
static CONTROL: Mutex<()> = Mutex::new(());

/// Start or stop the dead keys.
pub fn set_enabled(enabled: bool) {
    let _guard = CONTROL.lock().unwrap_or_else(|e| e.into_inner());
    let running = THREAD.load(Ordering::SeqCst);
    if enabled && running == 0 {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || run_hook_thread(tx));
        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(tid) if tid != 0 => THREAD.store(tid, Ordering::SeqCst),
            _ => log::error!("diacritic keys: keyboard hook could not be installed"),
        }
    } else if !enabled && running != 0 {
        unsafe {
            let _ = PostThreadMessageW(running, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        THREAD.store(0, Ordering::SeqCst);
    }
}

fn run_hook_thread(tx: mpsc::Sender<u32>) {
    unsafe {
        let hmod = GetModuleHandleW(None).ok().map(|h| HINSTANCE(h.0));
        let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0) {
            Ok(h) => h,
            Err(e) => {
                log::error!("SetWindowsHookExW: {e}");
                let _ = tx.send(0);
                return;
            }
        };
        // Make sure this thread has a message queue before anyone posts WM_QUIT to it.
        let mut msg = MSG::default();
        let _ = PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE);
        let _ = tx.send(GetCurrentThreadId());
        log::info!("diacritic keys enabled");

        // The hook is called while this loop waits for messages.
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}

        let _ = UnhookWindowsHookEx(hook);
        let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
        st.marks.clear();
        st.swallow_up.clear();
        log::info!("diacritic keys disabled");
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let injected = kb.flags.0 & LLKHF_INJECTED.0 != 0;
        if !injected {
            let msg = wparam.0 as u32;
            let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            if handle_key(kb.vkCode, down) {
                return LRESULT(1); // swallow
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn is_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
}

fn is_modifier(vk: u32) -> bool {
    [
        VK_SHIFT, VK_LSHIFT, VK_RSHIFT, VK_CONTROL, VK_LCONTROL, VK_RCONTROL, VK_MENU, VK_LMENU,
        VK_RMENU, VK_LWIN, VK_RWIN, VK_CAPITAL,
    ]
    .iter()
    .any(|m| m.0 as u32 == vk)
}

fn mark_for(vk: u32, shift: bool) -> Option<Mark> {
    let vk = VIRTUAL_KEY(vk as u16);
    Some(match (vk, shift) {
        (VK_OEM_PERIOD, false) => Mark::DotBelow,
        (VK_OEM_PERIOD, true) => Mark::DotAbove,
        (VK_OEM_MINUS, false) => Mark::Macron,
        (VK_OEM_MINUS, true) => Mark::LineBelow,
        (VK_OEM_COMMA, false) => Mark::Acute,
        (VK_OEM_COMMA, true) => Mark::Tilde,
        _ => return None,
    })
}

/// Returns true if the key event must be swallowed.
fn handle_key(vk: u32, down: bool) -> bool {
    let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());

    if !down {
        if let Some(i) = st.swallow_up.iter().position(|v| *v == vk) {
            st.swallow_up.remove(i);
            return true;
        }
        return false;
    }

    if st.since.is_some_and(|t| t.elapsed() > PENDING_TIMEOUT) {
        st.marks.clear();
        st.since = None;
    }

    let ctrl = is_down(VK_CONTROL);
    let alt = is_down(VK_MENU);
    let shift = is_down(VK_SHIFT);
    // AltGr arrives as LeftCtrl + RightAlt on European layouts.
    let altgr = (ctrl && alt) || is_down(VK_RMENU);

    let swallow = |st: &mut State, vk: u32| {
        if st.swallow_up.len() > 16 {
            st.swallow_up.remove(0);
        }
        st.swallow_up.push(vk);
    };

    if altgr {
        if let Some(mark) = mark_for(vk, shift) {
            st.marks.push(mark);
            st.since = Some(Instant::now());
            swallow(&mut st, vk);
            return true;
        }
        if !is_modifier(vk) {
            st.marks.clear(); // e.g. AltGr+Q (@): cancel, let it through
        }
        return false;
    }

    if st.marks.is_empty() || is_modifier(vk) {
        return false;
    }

    if vk == VK_ESCAPE.0 as u32 {
        st.marks.clear();
        swallow(&mut st, vk);
        return true;
    }

    // Letters A–Z (virtual-key codes follow the active layout, so Y/Z are right on QWERTZ).
    if (0x41..=0x5A).contains(&vk) && !ctrl && !alt {
        let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        let letter = vk as u8 as char;
        let base = if shift ^ caps { letter } else { letter.to_ascii_lowercase() };
        let text = compose(base, &st.marks);
        st.marks.clear();
        st.since = None;
        swallow(&mut st, vk);
        drop(st);
        super::input::type_text(&text);
        return true;
    }

    // Anything else: cancel the pending marks and let the key through.
    st.marks.clear();
    st.since = None;
    false
}
