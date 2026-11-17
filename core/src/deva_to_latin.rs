use unicode_normalization::UnicodeNormalization;

use crate::tables::*;
use crate::Options;

/// Devanagari → Latin (IAST or ISO 15919). Non-Devanagari text passes through.
pub fn to_latin(input: &str, o: &Options) -> String {
    // NFC also decomposes the precomposed nukta letters (U+0958–U+095F),
    // so क़ is always seen as क + ़ below.
    let s: Vec<char> = input.nfc().collect();
    let mut out = String::with_capacity(s.len() * 2);
    // A consonant was written and still awaits its inherent "a".
    let mut pending = false;
    let mut i = 0;

    while i < s.len() {
        let c = s[i];

        if let Some(base) = lookup(CONSONANTS, c) {
            if pending {
                out.push('a');
            }
            let mut latin = pick(base, o.scheme);
            if s.get(i + 1) == Some(&NUKTA) {
                i += 1;
                if o.nukta {
                    if let Some(n) = lookup(NUKTA_CONSONANTS, c) {
                        latin = pick(n, o.scheme);
                    }
                }
            }
            out.push_str(latin);
            pending = true;
        } else if let Some(m) = lookup(MATRAS, c) {
            out.push_str(pick(m, o.scheme));
            pending = false;
        } else if c == VIRAMA {
            pending = false;
        } else if c == NUKTA || c == ZWJ || c == ZWNJ {
            // Stray nukta or joiners: no Latin equivalent.
        } else if is_vowel_modifier(c) {
            if pending {
                out.push('a');
                pending = false;
            }
            out.push(c);
        } else {
            if pending {
                out.push('a');
                pending = false;
            }
            if let Some(v) = lookup(VOWELS, c) {
                out.push_str(pick(v, o.scheme));
            } else if let Some(d) = deva_digit(c) {
                if o.convert_digits {
                    out.push(d);
                } else {
                    out.push(c);
                }
            } else {
                match c {
                    ANUSVARA => out.push_str(pick(ANUSVARA_LATIN, o.scheme)),
                    VISARGA => out.push_str(VISARGA_LATIN),
                    CANDRABINDU => out.push_str(CANDRABINDU_LATIN),
                    AVAGRAHA => out.push_str(AVAGRAHA_LATIN),
                    DANDA => out.push('|'),
                    DOUBLE_DANDA => out.push_str("||"),
                    OM => out.push_str(pick(OM_LATIN, o.scheme)),
                    ABBREVIATION => out.push('.'),
                    _ => out.push(c),
                }
            }
        }
        i += 1;
    }
    if pending {
        out.push('a');
    }
    out.nfc().collect()
}
