//! Clipboard access: snapshot/restore of every format, reading/writing
//! Unicode text, and copying the current selection with Ctrl+C.
//!
//! Text we put on the clipboard is flagged so that Windows clipboard history
//! (Win+V), cloud clipboard and well-behaved clipboard managers ignore it.

use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};

use super::input;

const CF_UNICODETEXT: u32 = 13;

/// Formats whose handle is not an HGLOBAL (GDI objects, metafiles, owner-display).
/// Windows synthesises CF_BITMAP from CF_DIB, so images survive anyway.
fn is_global_memory_format(f: u32) -> bool {
    !matches!(f, 2 | 3 | 9 | 14 | 0x80 | 0x82 | 0x83 | 0x8E | 0x300..=0x3FF)
}

/// RAII clipboard lock; retries because other apps may hold it briefly.
struct Open;

impl Open {
    fn new() -> Option<Self> {
        for _ in 0..20 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(Open);
            }
            sleep(Duration::from_millis(15));
        }
        log::warn!("could not open the clipboard");
        None
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

fn read_global(handle: HANDLE) -> Option<Vec<u8>> {
    let h = HGLOBAL(handle.0);
    unsafe {
        let size = GlobalSize(h);
        let ptr = GlobalLock(h) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
        let _ = GlobalUnlock(h);
        Some(bytes)
    }
}

/// Must be called while the clipboard is open (after EmptyClipboard).
fn put(format: u32, bytes: &[u8]) -> bool {
    unsafe {
        let Ok(h) = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) else { return false };
        let ptr = GlobalLock(h) as *mut u8;
        if ptr.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        let _ = GlobalUnlock(h);
        // On success the system owns the memory.
        SetClipboardData(format, Some(HANDLE(h.0))).is_ok()
    }
}

fn put_private_flags() {
    unsafe {
        let zero = 0u32.to_ne_bytes();
        put(RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory")), &zero);
        put(RegisterClipboardFormatW(w!("CanUploadToCloudClipboard")), &zero);
        put(RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing")), &zero);
    }
}

/// A copy of everything that was on the clipboard.
pub struct Snapshot {
    items: Vec<(u32, Vec<u8>)>,
}

pub fn snapshot() -> Snapshot {
    let mut items = Vec::new();
    if let Some(_open) = Open::new() {
        let mut f = 0;
        loop {
            f = unsafe { EnumClipboardFormats(f) };
            if f == 0 {
                break;
            }
            if !is_global_memory_format(f) {
                continue;
            }
            if let Ok(h) = unsafe { GetClipboardData(f) } {
                if !h.is_invalid() {
                    if let Some(bytes) = read_global(h) {
                        items.push((f, bytes));
                    }
                }
            }
        }
    }
    Snapshot { items }
}

impl Snapshot {
    pub fn restore(self) {
        if let Some(_open) = Open::new() {
            unsafe {
                let _ = EmptyClipboard();
            }
            for (f, bytes) in &self.items {
                put(*f, bytes);
            }
            // Don't add the restored item to Win+V history a second time.
            put_private_flags();
        }
    }
}

pub fn sequence_number() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

pub fn get_text() -> Option<String> {
    let _open = Open::new()?;
    let h = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok()?;
    if h.is_invalid() {
        return None;
    }
    let bytes = read_global(h)?;
    let units: Vec<u16> = bytes.chunks_exact(2).map(|b| u16::from_ne_bytes([b[0], b[1]])).collect();
    let end = units.iter().position(|u| *u == 0).unwrap_or(units.len());
    Some(String::from_utf16_lossy(&units[..end]))
}

/// Put text on the clipboard (CRLF line endings), hidden from clipboard history.
pub fn set_text(text: &str) -> bool {
    let crlf = text.replace("\r\n", "\n").replace('\n', "\r\n");
    let mut units: Vec<u16> = crlf.encode_utf16().collect();
    units.push(0);
    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_ne_bytes()).collect();
    let Some(_open) = Open::new() else { return false };
    unsafe {
        let _ = EmptyClipboard();
    }
    let ok = put(CF_UNICODETEXT, &bytes);
    put_private_flags();
    ok
}

/// Send Ctrl+C and wait until the foreground app has put something on the
/// clipboard. Returns `None` if nothing was copied (e.g. no selection).
pub fn copy_selection(timeout: Duration) -> Option<String> {
    let before = sequence_number();
    input::send_copy();
    let start = Instant::now();
    while sequence_number() == before {
        if start.elapsed() > timeout {
            return None;
        }
        sleep(Duration::from_millis(10));
    }
    // Some apps write several formats one after another; give them a moment.
    sleep(Duration::from_millis(30));
    get_text().filter(|t| !t.is_empty())
}
