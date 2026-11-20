//! Diacritic composition for the AltGr dead keys: one or more marks are
//! pressed first, then the base letter (e.g. AltGr+. then r → ṛ).
//! Kept free of other dependencies so it can be tested on any platform.

use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// ṛ ṭ ḍ ṇ ṣ ṃ ḥ ḷ (AltGr + .)
    DotBelow,
    /// ṅ ṁ ġ ẏ (AltGr + Shift + .)
    DotAbove,
    /// ā ī ū ē ō (AltGr + -)
    Macron,
    /// ḻ ṉ ṟ ḵ (AltGr + Shift + -)
    LineBelow,
    /// ś (AltGr + ,)
    Acute,
    /// ñ (AltGr + Shift + ,)
    Tilde,
}

impl Mark {
    pub fn combining(self) -> char {
        match self {
            Mark::DotBelow => '\u{0323}',
            Mark::DotAbove => '\u{0307}',
            Mark::Macron => '\u{0304}',
            Mark::LineBelow => '\u{0331}',
            Mark::Acute => '\u{0301}',
            Mark::Tilde => '\u{0303}',
        }
    }
}

/// Base letter plus the pressed marks, normalised to NFC so the result is a
/// single precomposed character wherever Unicode has one (r + dot below +
/// macron → ṝ, in whichever order the marks were pressed). A mark pressed
/// twice counts once.
pub fn compose(base: char, marks: &[Mark]) -> String {
    let mut s = String::from(base);
    let mut seen: Vec<Mark> = Vec::new();
    for m in marks {
        if !seen.contains(m) {
            seen.push(*m);
            s.push(m.combining());
        }
    }
    s.nfc().collect()
}
