//! Mapping tables. Each Latin entry is `(IAST, ISO 15919)`.

use crate::Scheme;

pub type Pair = (&'static str, &'static str);

pub fn pick(p: Pair, s: Scheme) -> &'static str {
    match s {
        Scheme::Iast => p.0,
        Scheme::Iso15919 => p.1,
    }
}

pub const VIRAMA: char = '\u{094D}';
pub const NUKTA: char = '\u{093C}';
pub const ANUSVARA: char = '\u{0902}';
pub const VISARGA: char = '\u{0903}';
pub const CANDRABINDU: char = '\u{0901}';
pub const AVAGRAHA: char = '\u{093D}';
pub const DANDA: char = '\u{0964}';
pub const DOUBLE_DANDA: char = '\u{0965}';
pub const OM: char = '\u{0950}';
pub const ABBREVIATION: char = '\u{0970}';
pub const ZWNJ: char = '\u{200C}';
pub const ZWJ: char = '\u{200D}';

pub const ANUSVARA_LATIN: Pair = ("ṃ", "ṁ");
pub const VISARGA_LATIN: &str = "ḥ";
pub const CANDRABINDU_LATIN: &str = "m\u{0310}"; // m̐
pub const AVAGRAHA_LATIN: &str = "'";
pub const OM_LATIN: Pair = ("oṃ", "ōṁ");

/// Independent vowels: (Devanagari, (IAST, ISO)).
pub const VOWELS: &[(char, Pair)] = &[
    ('अ', ("a", "a")),
    ('आ', ("ā", "ā")),
    ('इ', ("i", "i")),
    ('ई', ("ī", "ī")),
    ('उ', ("u", "u")),
    ('ऊ', ("ū", "ū")),
    ('ऋ', ("ṛ", "r\u{0325}")),
    ('ॠ', ("ṝ", "r\u{0325}\u{0304}")),
    ('ऌ', ("ḷ", "l\u{0325}")),
    ('ॡ', ("ḹ", "l\u{0325}\u{0304}")),
    ('ऍ', ("ê", "ê")),
    ('ऎ', ("e", "e")),
    ('ए', ("e", "ē")),
    ('ऐ', ("ai", "ai")),
    ('ऑ', ("ô", "ô")),
    ('ऒ', ("o", "o")),
    ('ओ', ("o", "ō")),
    ('औ', ("au", "au")),
];

/// Dependent vowel signs (mātrās).
pub const MATRAS: &[(char, Pair)] = &[
    ('\u{093E}', ("ā", "ā")),
    ('\u{093F}', ("i", "i")),
    ('\u{0940}', ("ī", "ī")),
    ('\u{0941}', ("u", "u")),
    ('\u{0942}', ("ū", "ū")),
    ('\u{0943}', ("ṛ", "r\u{0325}")),
    ('\u{0944}', ("ṝ", "r\u{0325}\u{0304}")),
    ('\u{0962}', ("ḷ", "l\u{0325}")),
    ('\u{0963}', ("ḹ", "l\u{0325}\u{0304}")),
    ('\u{0945}', ("ê", "ê")),
    ('\u{0946}', ("e", "e")),
    ('\u{0947}', ("e", "ē")),
    ('\u{0948}', ("ai", "ai")),
    ('\u{0949}', ("ô", "ô")),
    ('\u{094A}', ("o", "o")),
    ('\u{094B}', ("o", "ō")),
    ('\u{094C}', ("au", "au")),
];

pub const CONSONANTS: &[(char, Pair)] = &[
    ('क', ("k", "k")),
    ('ख', ("kh", "kh")),
    ('ग', ("g", "g")),
    ('घ', ("gh", "gh")),
    ('ङ', ("ṅ", "ṅ")),
    ('च', ("c", "c")),
    ('छ', ("ch", "ch")),
    ('ज', ("j", "j")),
    ('झ', ("jh", "jh")),
    ('ञ', ("ñ", "ñ")),
    ('ट', ("ṭ", "ṭ")),
    ('ठ', ("ṭh", "ṭh")),
    ('ड', ("ḍ", "ḍ")),
    ('ढ', ("ḍh", "ḍh")),
    ('ण', ("ṇ", "ṇ")),
    ('त', ("t", "t")),
    ('थ', ("th", "th")),
    ('द', ("d", "d")),
    ('ध', ("dh", "dh")),
    ('न', ("n", "n")),
    ('ऩ', ("ṉ", "ṉ")),
    ('प', ("p", "p")),
    ('फ', ("ph", "ph")),
    ('ब', ("b", "b")),
    ('भ', ("bh", "bh")),
    ('म', ("m", "m")),
    ('य', ("y", "y")),
    ('र', ("r", "r")),
    ('ऱ', ("ṟ", "ṟ")),
    ('ल', ("l", "l")),
    ('ळ', ("ḷ", "ḷ")),
    ('ऴ', ("ḻ", "ḻ")),
    ('व', ("v", "v")),
    ('श', ("ś", "ś")),
    ('ष', ("ṣ", "ṣ")),
    ('स', ("s", "s")),
    ('ह', ("h", "h")),
];

/// Consonant + nukta (Hindi/Urdu sounds), keyed by the base consonant.
pub const NUKTA_CONSONANTS: &[(char, Pair)] = &[
    ('क', ("q", "q")),
    ('ख', ("k\u{035F}h", "k\u{035F}h")), // k͟h
    ('ग', ("ġ", "ġ")),
    ('ज', ("z", "z")),
    ('ड', ("ṛ", "ṛ")),
    ('ढ', ("ṛh", "ṛh")),
    ('फ', ("f", "f")),
    ('य', ("ẏ", "ẏ")),
];

pub fn lookup(table: &[(char, Pair)], c: char) -> Option<Pair> {
    table.iter().find(|(d, _)| *d == c).map(|(_, p)| *p)
}

pub fn deva_digit(c: char) -> Option<char> {
    match c {
        '\u{0966}'..='\u{096F}' => char::from_digit(c as u32 - 0x0966, 10),
        _ => None,
    }
}

pub fn latin_digit(c: char) -> Option<char> {
    c.to_digit(10).filter(|_| c.is_ascii_digit()).and_then(|d| char::from_u32(0x0966 + d))
}

/// Vedic accent and other combining marks that attach to a vowel.
pub fn is_vowel_modifier(c: char) -> bool {
    matches!(c, '\u{0951}'..='\u{0954}' | '\u{1CD0}'..='\u{1CF9}' | '\u{A8E0}'..='\u{A8F1}')
}
