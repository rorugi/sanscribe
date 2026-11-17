use unicode_normalization::UnicodeNormalization;

use crate::tables::*;
use crate::{Options, Scheme};

#[derive(Clone, Debug)]
enum Tok {
    Consonant(String),
    Vowel { independent: char, matra: Option<char> },
    /// IAST "ḷ": vowel ऌ, or consonant ळ before a vowel (agnim īḷe).
    LDot,
    Sign(char),
}

fn nfc_lower(s: &str) -> Vec<char> {
    s.nfc().flat_map(char::to_lowercase).collect()
}

fn build_table(o: &Options) -> Vec<(Vec<char>, Tok)> {
    let mut t: Vec<(Vec<char>, Tok)> = Vec::new();
    let mut add = |key: &str, tok: Tok| t.push((nfc_lower(key), tok));

    let vowels: &[(&str, char, Option<char>)] = &[
        ("a", 'अ', None),
        ("ā", 'आ', Some('\u{093E}')),
        ("i", 'इ', Some('\u{093F}')),
        ("ī", 'ई', Some('\u{0940}')),
        ("u", 'उ', Some('\u{0941}')),
        ("ū", 'ऊ', Some('\u{0942}')),
        ("r\u{0325}", 'ऋ', Some('\u{0943}')),
        ("r\u{0325}\u{0304}", 'ॠ', Some('\u{0944}')),
        ("ṝ", 'ॠ', Some('\u{0944}')),
        ("l\u{0325}", 'ऌ', Some('\u{0962}')),
        ("l\u{0325}\u{0304}", 'ॡ', Some('\u{0963}')),
        ("ḹ", 'ॡ', Some('\u{0963}')),
        ("e", 'ए', Some('\u{0947}')),
        ("ē", 'ए', Some('\u{0947}')),
        ("ai", 'ऐ', Some('\u{0948}')),
        ("o", 'ओ', Some('\u{094B}')),
        ("ō", 'ओ', Some('\u{094B}')),
        ("au", 'औ', Some('\u{094C}')),
        ("ê", 'ऍ', Some('\u{0945}')),
        ("ô", 'ऑ', Some('\u{0949}')),
    ];
    for (k, ind, m) in vowels {
        add(k, Tok::Vowel { independent: *ind, matra: *m });
    }

    for (deva, (iast, iso)) in CONSONANTS {
        if *deva == 'ळ' {
            continue; // handled below
        }
        add(iast, Tok::Consonant(deva.to_string()));
        if iso != iast {
            add(iso, Tok::Consonant(deva.to_string()));
        }
    }
    add("w", Tok::Consonant("व".into()));

    let iso_nukta_r = o.scheme == Scheme::Iso15919 && o.nukta;
    if iso_nukta_r {
        add("ṛ", Tok::Consonant("ड\u{093C}".into()));
        add("ṛh", Tok::Consonant("ढ\u{093C}".into()));
    } else {
        add("ṛ", Tok::Vowel { independent: 'ऋ', matra: Some('\u{0943}') });
    }
    match o.scheme {
        Scheme::Iast => add("ḷ", Tok::LDot),
        Scheme::Iso15919 => add("ḷ", Tok::Consonant("ळ".into())),
    }

    if o.nukta {
        for (key, base) in [
            ("q", 'क'),
            ("k\u{035F}h", 'ख'),
            ("ḵh", 'ख'),
            ("ġ", 'ग'),
            ("z", 'ज'),
            ("f", 'फ'),
            ("ẏ", 'य'),
        ] {
            add(key, Tok::Consonant(format!("{base}{NUKTA}")));
        }
    }

    add("ṃ", Tok::Sign(ANUSVARA));
    add("ṁ", Tok::Sign(ANUSVARA));
    add("ḥ", Tok::Sign(VISARGA));
    add(CANDRABINDU_LATIN, Tok::Sign(CANDRABINDU));

    // Longest keys first so that greedy matching prefers "kh" over "k".
    t.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
    t
}

fn match_at<'a>(table: &'a [(Vec<char>, Tok)], s: &[char], i: usize) -> Option<(usize, &'a Tok)> {
    table
        .iter()
        .find(|(k, _)| s.len() >= i + k.len() && s[i..i + k.len()] == k[..])
        .map(|(k, t)| (k.len(), t))
}

fn is_letter(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_alphabetic() || ('\u{0300}'..='\u{036F}').contains(c))
}

/// Latin (IAST or ISO 15919, case-insensitive) → Devanagari.
/// Characters that are not part of the scheme pass through unchanged.
pub fn to_devanagari(input: &str, o: &Options) -> String {
    let orig: Vec<char> = input.nfc().collect();
    // Lower-cased copy with the same length (multi-char lowercase mappings keep the original char).
    let s: Vec<char> = orig
        .iter()
        .map(|c| {
            let mut l = c.to_lowercase();
            match (l.next(), l.next()) {
                (Some(x), None) => x,
                _ => *c,
            }
        })
        .collect();
    let table = build_table(o);
    let om_keys: Vec<Vec<char>> = ["oṃ", "oṁ", "ōṃ", "ōṁ"].iter().map(|k| nfc_lower(k)).collect();

    let mut out = String::with_capacity(input.len() * 2);
    let mut after_consonant = false;
    let mut i = 0;

    let close = |out: &mut String, after: &mut bool| {
        if *after {
            out.push(VIRAMA);
            *after = false;
        }
    };

    while i < s.len() {
        // ॐ as a standalone word.
        if !is_letter(i.checked_sub(1).and_then(|p| s.get(p))) {
            if let Some(k) = om_keys.iter().find(|k| s[i..].starts_with(k)) {
                if !is_letter(s.get(i + k.len())) {
                    close(&mut out, &mut after_consonant);
                    out.push(OM);
                    i += k.len();
                    continue;
                }
            }
        }

        let c = s[i];

        // Avagraha: an apostrophe after e/o/ā and before a letter (so 'ham → सोऽहम्).
        if c == '\'' || c == '\u{2019}' {
            let prev = s[..i].iter().rev().find(|c| **c != ' ');
            if matches!(prev, Some('e' | 'o' | 'ē' | 'ō' | 'ā')) && is_letter(s.get(i + 1)) {
                close(&mut out, &mut after_consonant);
                if out.ends_with(' ') {
                    out.pop();
                }
                out.push(AVAGRAHA);
                i += 1;
                continue;
            }
        }

        if c == '|' {
            close(&mut out, &mut after_consonant);
            if s.get(i + 1) == Some(&'|') {
                out.push(DOUBLE_DANDA);
                i += 2;
            } else {
                out.push(DANDA);
                i += 1;
            }
            continue;
        }

        if o.convert_digits {
            if let Some(d) = latin_digit(c) {
                close(&mut out, &mut after_consonant);
                out.push(d);
                i += 1;
                continue;
            }
        }

        match match_at(&table, &s, i) {
            Some((len, tok)) => {
                let tok = match tok {
                    Tok::LDot => {
                        // Consonant ळ when a vowel (or the "h" of ḷh) follows.
                        let next = s.get(i + len);
                        let vowel_next = matches!(
                            match_at(&table, &s, i + len),
                            Some((_, Tok::Vowel { .. })) | Some((_, Tok::LDot))
                        );
                        if vowel_next || next == Some(&'h') {
                            Tok::Consonant("ळ".into())
                        } else {
                            Tok::Vowel { independent: 'ऌ', matra: Some('\u{0962}') }
                        }
                    }
                    t => t.clone(),
                };
                match tok {
                    Tok::Consonant(d) => {
                        if after_consonant {
                            out.push(VIRAMA);
                        }
                        out.push_str(&d);
                        after_consonant = true;
                    }
                    Tok::Vowel { independent, matra } => {
                        if after_consonant {
                            if let Some(m) = matra {
                                out.push(m);
                            }
                        } else {
                            out.push(independent);
                        }
                        after_consonant = false;
                    }
                    Tok::Sign(d) => {
                        close(&mut out, &mut after_consonant);
                        out.push(d);
                    }
                    Tok::LDot => unreachable!(),
                }
                i += len;
            }
            None => {
                close(&mut out, &mut after_consonant);
                out.push(orig[i]);
                i += 1;
            }
        }
    }
    close(&mut out, &mut after_consonant);
    out.nfc().collect()
}
