use unicode_normalization::UnicodeNormalization;

use crate::{transliterate, Options};

/// Remembers the last conversion so that toggling back returns exactly the
/// original text, even where Latin → Devanagari would be ambiguous
/// (e.g. अइ and ऐ are both "ai").
#[derive(Default, Debug)]
pub struct Session {
    /// (text we produced, text it came from)
    last_toggle: Option<(String, String)>,
    /// The last text produced by "insert alongside".
    last_alongside: Option<String>,
}

/// Compare selections independently of line-ending style and normalization:
/// applications often hand back "\r\n" for text we inserted with "\n".
fn key(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n").trim_end_matches('\n').nfc().collect()
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hotkey 1: Devanagari ↔ Latin. Returns `None` if nothing to do.
    pub fn toggle(&mut self, text: &str, opts: &Options) -> Option<String> {
        if let Some((produced, source)) = &self.last_toggle {
            if key(text) == key(produced) {
                let (produced, source) = (produced.clone(), source.clone());
                self.last_toggle = Some((source.clone(), produced));
                return Some(source);
            }
        }
        let out = transliterate(text, opts)?;
        self.last_toggle = Some((out.clone(), text.to_string()));
        Some(out)
    }

    /// Hotkey 2: keep the original and add the transliteration using
    /// `template`, which may contain `{original}` and `{result}`.
    /// Pressing it again on its own output does nothing.
    pub fn alongside(&mut self, text: &str, template: &str, opts: &Options) -> Option<String> {
        if self.last_alongside.as_deref().map(key) == Some(key(text)) {
            return None;
        }
        // Keep surrounding whitespace outside the template: Word, for
        // example, includes the trailing space when a word is double-clicked.
        let core = text.trim();
        let start = text.find(core).unwrap_or(0);
        let (lead, trail) = (&text[..start], &text[start + core.len()..]);
        let result = transliterate(core, opts)?;
        let out = format!(
            "{lead}{}{trail}",
            template.replace("{result}", &result).replace("{original}", core)
        );
        self.last_alongside = Some(out.clone());
        Some(out)
    }

    pub fn forget(&mut self) {
        *self = Self::default();
    }
}
