use sanscribe_core::{compose, Mark::*};

#[test]
fn all_requested_letters() {
    // āīūṛṝḷṅñṭḍṇśṣṃḥ
    let cases: &[(char, &[sanscribe_core::Mark], &str)] = &[
        ('a', &[Macron], "ā"),
        ('i', &[Macron], "ī"),
        ('u', &[Macron], "ū"),
        ('r', &[DotBelow], "ṛ"),
        ('r', &[DotBelow, Macron], "ṝ"),
        ('r', &[Macron, DotBelow], "ṝ"),
        ('l', &[DotBelow], "ḷ"),
        ('l', &[DotBelow, Macron], "ḹ"),
        ('n', &[DotAbove], "ṅ"),
        ('n', &[Tilde], "ñ"),
        ('t', &[DotBelow], "ṭ"),
        ('d', &[DotBelow], "ḍ"),
        ('n', &[DotBelow], "ṇ"),
        ('s', &[Acute], "ś"),
        ('s', &[DotBelow], "ṣ"),
        ('m', &[DotBelow], "ṃ"),
        ('h', &[DotBelow], "ḥ"),
    ];
    for (base, marks, want) in cases {
        let got = compose(*base, marks);
        assert_eq!(got, *want, "{base} {marks:?}");
        assert_eq!(got.chars().count(), 1, "{want} should be one precomposed character");
    }
}

#[test]
fn other_letters_and_capitals() {
    assert_eq!(compose('m', &[DotAbove]), "ṁ");
    assert_eq!(compose('l', &[LineBelow]), "ḻ");
    assert_eq!(compose('e', &[Macron]), "ē");
    assert_eq!(compose('A', &[Macron]), "Ā");
    assert_eq!(compose('S', &[Acute]), "Ś");
    assert_eq!(compose('R', &[DotBelow, Macron]), "Ṝ");
    // pressing the same mark twice counts once
    assert_eq!(compose('a', &[Macron, Macron]), "ā");
    // no precomposed form: stays base + combining mark
    assert_eq!(compose('k', &[DotAbove]), "k\u{0307}");
}
