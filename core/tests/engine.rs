use sanscribe_core::*;

fn iast() -> Options {
    Options::default()
}
fn iso() -> Options {
    Options { scheme: Scheme::Iso15919, ..Options::default() }
}
fn with_nukta(mut o: Options) -> Options {
    o.nukta = true;
    o
}

/// (Devanagari, IAST) pairs that must convert exactly in both directions.
const IAST_PAIRS: &[(&str, &str)] = &[
    ("संस्कृतम्", "saṃskṛtam"),
    ("धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः ।", "dharmakṣetre kurukṣetre samavetā yuyutsavaḥ |"),
    ("मामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय ॥१॥", "māmakāḥ pāṇḍavāścaiva kimakurvata sañjaya ||1||"),
    ("सोऽहम्", "so'ham"),
    ("अग्निमीळे पुरोहितं", "agnimīḷe purohitaṃ"),
    ("कॢप्त", "kḷpta"),
    ("ॐ नमः शिवाय", "oṃ namaḥ śivāya"),
    ("हँस", "ham̐sa"),
    ("ज्ञानम्", "jñānam"),
    ("कृष्ण", "kṛṣṇa"),
    ("भगवद्गीता", "bhagavadgītā"),
    ("पितॄन्", "pitṝn"),
    ("औषधम्", "auṣadham"),
    ("ऐरावत", "airāvata"),
    ("ईश्वरः", "īśvaraḥ"),
    ("ऊर्ध्व", "ūrdhva"),
    ("ऋषिः", "ṛṣiḥ"),
    ("शङ्कराचार्य", "śaṅkarācārya"),
    ("दुःख", "duḥkha"),
    ("अद्भुत", "adbhuta"),
    ("ब्रह्मसूत्र", "brahmasūtra"),
    ("स्त्री", "strī"),
    ("कार्त्स्न्य", "kārtsnya"),
];

#[test]
fn devanagari_to_iast() {
    for (d, l) in IAST_PAIRS {
        assert_eq!(to_latin(d, &iast()), *l, "{d}");
    }
}

#[test]
fn iast_to_devanagari() {
    for (d, l) in IAST_PAIRS {
        assert_eq!(to_devanagari(l, &iast()), *d, "{l}");
    }
}

#[test]
fn iso_15919() {
    assert_eq!(to_latin("संस्कृतम्", &iso()), "saṁskr̥tam");
    assert_eq!(to_latin("देवो", &iso()), "dēvō");
    assert_eq!(to_latin("ॐ", &iso()), "ōṁ");
    assert_eq!(to_latin("पितॄन्", &iso()), "pitr̥̄n");
    assert_eq!(to_latin("कॢप्त", &iso()), "kl̥pta");
    assert_eq!(to_devanagari("saṁskr̥tam", &iso()), "संस्कृतम्");
    assert_eq!(to_devanagari("dēvō", &iso()), "देवो");
    assert_eq!(to_devanagari("pitr̥̄n", &iso()), "पितॄन्");
    assert_eq!(to_devanagari("agnimīḷe", &iso()), "अग्निमीळे");
}

#[test]
fn nukta_letters() {
    // Precomposed U+095B and decomposed ज + ़ behave the same.
    assert_eq!(to_latin("\u{095B}मीन", &with_nukta(iast())), "zamīna");
    assert_eq!(to_latin("ज\u{093C}मीन", &with_nukta(iast())), "zamīna");
    assert_eq!(to_latin("ज़मीन", &iast()), "jamīna", "nukta off: ignored");
    assert_eq!(to_latin("क़लम", &with_nukta(iast())), "qalama");
    assert_eq!(to_latin("ख़ून", &with_nukta(iast())), "k͟hūna");
    assert_eq!(to_latin("सड़क", &with_nukta(iso())), "saṛaka");
    assert_eq!(to_latin("पढ़", &with_nukta(iso())), "paṛha");
    assert_eq!(to_latin("फ़", &with_nukta(iast())), "fa");
    // Reverse
    assert_eq!(to_devanagari("zamīna", &with_nukta(iast())), "ज\u{093C}मीन");
    assert_eq!(to_devanagari("saṛaka", &with_nukta(iso())), "सड\u{093C}क");
    assert_eq!(to_devanagari("saṛaka", &with_nukta(iast())), "सऋअक", "IAST: ṛ stays the vowel");
}

#[test]
fn digits() {
    let keep = Options { convert_digits: false, ..iast() };
    assert_eq!(to_latin("॥१२॥", &iast()), "||12||");
    assert_eq!(to_latin("॥१२॥", &keep), "||१२||");
    assert_eq!(to_devanagari("||12||", &iast()), "॥१२॥");
    assert_eq!(to_devanagari("||12||", &keep), "॥12॥");
}

#[test]
fn mixed_text_and_input_variants() {
    assert_eq!(to_latin("The word संस्कृतम्.", &iast()), "The word saṃskṛtam.");
    // Capitals and decomposed (NFD) input
    assert_eq!(to_devanagari("Saṃskṛtam", &iast()), "संस्कृतम्");
    assert_eq!(to_devanagari("sam\u{0323}skr\u{0323}tam", &iast()), "संस्कृतम्");
    // ISO anusvara accepted in IAST mode and vice versa
    assert_eq!(to_devanagari("saṁskṛtam", &iast()), "संस्कृतम्");
    // avagraha written with a space and typographic apostrophe
    assert_eq!(to_devanagari("so ’ham", &iast()), "सोऽहम्");
    // quotes are not avagraha
    assert_eq!(to_devanagari("'yoga'", &iast()), "'योग'");
    // multi-line
    assert_eq!(to_latin("राम\r\nसीता", &iast()), "rāma\r\nsītā");
    // joiners are dropped
    assert_eq!(to_latin("क्\u{200D}ष", &iast()), "kṣa");
}

#[test]
fn detection() {
    assert_eq!(detect_direction("संस्कृतम्"), Some(Direction::ToLatin));
    assert_eq!(detect_direction("saṃskṛtam"), Some(Direction::ToDevanagari));
    assert_eq!(detect_direction("123 !"), None);
    assert_eq!(transliterate("  ", &iast()), None);
}

#[test]
fn toggle_round_trip_is_exact() {
    let mut s = Session::new();
    let o = iast();
    assert_eq!(s.toggle("संस्कृतम्", &o).as_deref(), Some("saṃskṛtam"));
    assert_eq!(s.toggle("saṃskṛtam", &o).as_deref(), Some("संस्कृतम्"));
    assert_eq!(s.toggle("संस्कृतम्", &o).as_deref(), Some("saṃskṛtam"));

    // Ambiguous: अइ and ऐ are both "ai" — the session restores the original.
    let mut s = Session::new();
    assert_eq!(s.toggle("अइ", &o).as_deref(), Some("ai"));
    assert_eq!(s.toggle("ai", &o).as_deref(), Some("अइ"));
    // Without history, "ai" means ऐ.
    assert_eq!(Session::new().toggle("ai", &o).as_deref(), Some("ऐ"));

    // Application returned CRLF for text we produced with LF.
    let mut s = Session::new();
    s.toggle("राम\nसीता", &o);
    assert_eq!(s.toggle("rāma\r\nsītā", &o).as_deref(), Some("राम\nसीता"));
}

#[test]
fn alongside() {
    let o = iast();
    let mut s = Session::new();
    let t = "{original} ({result})";
    let out = s.alongside("संस्कृतम्", t, &o).unwrap();
    assert_eq!(out, "संस्कृतम् (saṃskṛtam)");
    assert_eq!(s.alongside(&out, t, &o), None, "second press does nothing");
    assert_eq!(
        Session::new().alongside("saṃskṛtam", t, &o).as_deref(),
        Some("saṃskṛtam (संस्कृतम्)")
    );
    // trailing space from a double-clicked word stays outside
    assert_eq!(
        Session::new().alongside("संस्कृतम् ", t, &o).as_deref(),
        Some("संस्कृतम् (saṃskṛtam) ")
    );
    assert_eq!(
        Session::new().alongside("राम", "{original} / {result}", &o).as_deref(),
        Some("राम / rāma")
    );
}
