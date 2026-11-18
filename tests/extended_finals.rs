use bamboo_core::{Config, Engine, InputMethod, Mode};

fn typed_with(finals: bool, auto_correct: bool, free_tone_marking: bool, keys: &str) -> String {
    let config = Config::builder()
        .extended_finals(finals)
        .auto_correct(auto_correct)
        .free_tone_marking(free_tone_marking)
        .build();
    let mut engine = Engine::with_config(InputMethod::telex(), config);
    engine.process_str(keys, Mode::Vietnamese);
    engine.output().into_owned()
}

fn typed(finals: bool, keys: &str) -> String {
    typed_with(finals, true, true, keys)
}

#[test]
fn off_by_default() {
    assert!(!Config::default().extended_finals);
    for keys in ["nhuwxg", "nhuwgx", "khosk", "mifnk", "book"] {
        assert_eq!(typed(false, keys), keys);
    }
}

#[test]
fn a_tone_lands_before_or_after_the_final() {
    for (before, after, want) in [
        ("nhuwxg", "nhuwgx", "nhữg"),
        ("cuxg", "cugx", "cũg"),
        ("huowsg", "huowgs", "hướg"),
        ("khosk", "khoks", "khók"),
        ("mifnk", "minkf", "mìnk"),
        ("mifh", "mihf", "mìh"),
    ] {
        assert_eq!(typed(true, before), want, "{before}");
        assert_eq!(typed(true, after), want, "{after}");
    }
}

#[test]
fn uppercase_keeps_the_final() {
    assert_eq!(typed(true, "NHUWGX"), "NHỮG");
    assert_eq!(typed(true, "Minkf"), "Mìnk");
}

/// Each pair types the same word with a standard final and with its teen code final.
const PAIRS: &[(&str, &str, &str, &str)] = &[
    ("nhuwngx", "nhuwgx", "ng", "g"),
    ("nhuwxng", "nhuwxg", "ng", "g"),
    ("huowngs", "huowgs", "ng", "g"),
    ("huowsng", "huowsg", "ng", "g"),
    ("nguwowngf", "nguwowgf", "ng", "g"),
    ("khoongf", "khoogf", "ng", "g"),
    // `ing` and `êng` are not rimes, so `ig` and `êg` are not either.
    ("tringj", "trigj", "ng", "g"),
    ("keeng", "keeg", "ng", "g"),
    ("eengs", "eegs", "ng", "g"),
    ("khocs", "khoks", "c", "k"),
    ("khofc", "khofk", "c", "k"),
    ("vieecj", "vieekj", "c", "k"),
    ("minhf", "minkf", "nh", "nk"),
    ("minhf", "mihf", "nh", "h"),
    ("nhanh", "nhank", "nh", "nk"),
    // `ia` and `ai` take no final; the teen code finals follow the standard ones here too.
    ("miasnh", "miasnk", "nh", "nk"),
    ("mianhs", "mianks", "nh", "nk"),
    ("maisnh", "maisnk", "nh", "nk"),
    ("mainhs", "mainks", "nh", "nk"),
];

#[test]
fn teen_code_finals_follow_the_standard_ones() {
    for auto_correct in [true, false] {
        for free in [true, false] {
            for &(standard, teen, std_final, teen_final) in PAIRS {
                let out = typed_with(true, auto_correct, free, standard);
                let i = out.rfind(std_final).unwrap();
                let want = format!("{}{teen_final}{}", &out[..i], &out[i + std_final.len()..]);
                assert_eq!(
                    typed_with(true, auto_correct, free, teen),
                    want,
                    "{teen} auto_correct={auto_correct} free={free}"
                );
            }
        }
    }
}

#[test]
fn ia_and_ai_match_the_standard_finals() {
    // With auto-correct off, a tone typed after an invalid final is refused, as for `nh`.
    assert_eq!(typed_with(true, false, true, "miasnk"), "míank");
    assert_eq!(typed_with(true, false, true, "mianks"), "mianks");
    assert_eq!(typed_with(true, false, true, "maisnk"), "máink");
    assert_eq!(typed_with(true, false, true, "mainks"), "mainks");
}

#[test]
fn english_words_ending_in_these_finals_change() {
    assert_eq!(typed(true, "book"), "bôk");
    assert_eq!(typed(false, "book"), "book");
}

#[test]
fn onsets_and_finals_are_separate() {
    let config = Config::builder().extended_onsets(false).extended_finals(true).build();
    let mut engine = Engine::with_config(InputMethod::telex(), config);
    engine.process_str("jaayj", Mode::Vietnamese);
    assert_eq!(engine.output(), "jaayj");
    engine.reset();
    engine.process_str("nhuwgx", Mode::Vietnamese);
    assert_eq!(engine.output(), "nhữg");
}
