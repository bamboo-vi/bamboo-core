use bamboo_core::{Engine, InputMethod, Mode};

fn check(typed: &str, want: &str, want_valid: bool) {
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str(typed, Mode::Vietnamese);
    assert_eq!(engine.output(), want, "typed {typed:?}");
    assert_eq!(engine.is_valid_word(), want_valid, "is_valid_word() for {typed:?}");
}

#[test]
fn any_onset_pairs_with_oa_breve() {
    check("boawjm", "boặm", true);
    check("moawsm", "moắm", true);
    check("noawfm", "noằm", true);
    check("toawst", "toắt", true);
    check("nhoawngf", "nhoằng", true);
}

#[test]
fn onset_pairs_that_already_worked() {
    check("khoawsm", "khoắm", true);
    check("hoawcs", "hoắc", true);
}

#[test]
fn kr_onset_for_proper_names() {
    check("kroong", "krông", true);
}

#[test]
fn rimes_with_final_k() {
    check("buks", "búk", true);
    check("tawsk", "tắk", true);
    // "a" + "k" is still not a rime.
    check("tak", "tak", false);
}

#[test]
fn rime_ueu() {
    check("khueeuf", "khuều", true);
}
