//! Sanscribe core: Devanagari <-> Latin (IAST / ISO 15919) transliteration.
//!
//! The engine follows Sanskrit orthography (inherent vowel always written,
//! no Hindi schwa deletion). It is pure Rust with no OS dependencies so it
//! can be unit-tested on any platform.

mod deva_to_latin;
mod latin_to_deva;
mod session;
mod tables;

use serde::{Deserialize, Serialize};

pub use deva_to_latin::to_latin;
pub use latin_to_deva::to_devanagari;
pub use session::Session;

/// Latin transliteration standard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// International Alphabet of Sanskrit Transliteration (ṃ, ṛ, e, o).
    #[default]
    Iast,
    /// ISO 15919 (ṁ, r̥, ē, ō).
    Iso15919,
}

/// User-configurable transliteration options.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Options {
    pub scheme: Scheme,
    /// Transliterate nukta letters (क़ ख़ ग़ ज़ ड़ ढ़ फ़ य़) as q k͟h ġ z ṛ ṛh f ẏ.
    /// When off, the nukta is ignored (क़ → k), as in Sanskrit text.
    pub nukta: bool,
    /// Convert digits between Devanagari (०–९) and Western (0–9).
    pub convert_digits: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { scheme: Scheme::Iast, nukta: false, convert_digits: true }
    }
}

/// Which way a piece of text should be transliterated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    ToLatin,
    ToDevanagari,
}

/// True for characters in the Devanagari, Devanagari Extended and Vedic Extensions blocks.
pub fn is_devanagari(c: char) -> bool {
    matches!(c, '\u{0900}'..='\u{097F}' | '\u{A8E0}'..='\u{A8FF}' | '\u{1CD0}'..='\u{1CFF}')
}

/// Text containing any Devanagari is converted to Latin (other text passes
/// through untouched); otherwise Latin letters are converted to Devanagari.
pub fn detect_direction(text: &str) -> Option<Direction> {
    if text.chars().any(is_devanagari) {
        Some(Direction::ToLatin)
    } else if text.chars().any(|c| c.is_alphabetic()) {
        Some(Direction::ToDevanagari)
    } else {
        None
    }
}

/// Transliterate in the detected direction. Returns `None` when there is
/// nothing to transliterate or the result would be identical.
pub fn transliterate(text: &str, opts: &Options) -> Option<String> {
    let out = match detect_direction(text)? {
        Direction::ToLatin => to_latin(text, opts),
        Direction::ToDevanagari => to_devanagari(text, opts),
    };
    (out != text).then_some(out)
}
