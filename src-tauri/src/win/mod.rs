//! Windows-specific integration: reading the selection, the clipboard and
//! synthetic keyboard input.

pub mod clipboard;
pub mod deadkeys;
pub mod input;
pub mod uia;

use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONWARNING;

/// Short system sound to signal "nothing to do / didn't work".
pub fn beep() {
    unsafe {
        let _ = MessageBeep(MB_ICONWARNING);
    }
}

/// Initialises COM (multithreaded) for the current thread and uninitialises it on drop.
pub struct ComGuard(bool);

impl ComGuard {
    pub fn new() -> Self {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        ComGuard(hr.is_ok())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}
