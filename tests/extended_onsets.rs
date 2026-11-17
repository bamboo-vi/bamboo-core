use bamboo_core::{Config, Engine, InputMethod, LastWordForm, Mode, OutputOptions};

fn config(extended_onsets: bool, auto_correct: bool) -> Config {
    Config::builder().extended_onsets(extended_onsets).auto_correct(auto_correct).build()
}

fn new_engine(im: InputMethod, extended_onsets: bool) -> Engine {
    Engine::with_config(im, config(extended_onsets, true))
}

/// Types `keys` in Telex: `<` deletes the last character, `+` and `-` turn extended onsets on
/// and off, and `!` restores the raw keys of the word.
fn typed_with(start: bool, auto_correct: bool, keys: &str) -> String {
    let mut engine = Engine::with_config(InputMethod::telex(), config(start, auto_correct));
    for key in keys.chars() {
        match key {
            '<' => engine.remove_last_output_char(),
            '+' => engine.set_config(config(true, auto_correct)),
            '-' => engine.set_config(config(false, auto_correct)),
            '!' => engine.restore_last_word_as(LastWordForm::RawKeystrokes),
            _ => engine.process_key(key, Mode::Vietnamese),
        }
    }
    engine.output().into_owned()
}

fn typed(start: bool, keys: &str) -> String {
    typed_with(start, true, keys)
}

fn typed_in(im: InputMethod, extended_onsets: bool, keys: &str) -> String {
    let mut engine = new_engine(im, extended_onsets);
    engine.process_str(keys, Mode::Vietnamese);
    engine.output().into_owned()
}

#[test]
fn on_by_default() {
    assert!(Config::default().extended_onsets);
    assert!(Engine::new(InputMethod::telex()).config().extended_onsets);
    assert_eq!(typed_in(InputMethod::telex(), true, "jaayj"), "jậy");
}

#[test]
fn on_takes_f_j_and_w() {
    for (keys, want) in [("faayj", "fậy"), ("jaayj", "jậy"), ("waayj", "wậy"), ("fair", "fải")]
    {
        assert_eq!(typed(true, keys), want);
    }
    assert_eq!(typed(true, "just"), "jút");
    assert_eq!(typed(true, "ferry"), "fery");
}

#[test]
fn off_keeps_f_j_and_w_as_typed() {
    for keys in ["faayj", "jaayj", "waayj", "fair", "just", "jussive", "ferry"] {
        assert_eq!(typed(false, keys), keys);
    }
}

#[test]
fn off_still_takes_z_and_vietnamese() {
    for on in [true, false] {
        assert_eq!(typed(on, "zaayj"), "zậy");
        assert_eq!(typed(on, "vieetj"), "việt");
        assert_eq!(typed(on, "tieengs"), "tiếng");
    }
}

#[test]
fn off_reports_the_word_invalid() {
    // Front ends that keep auto-correct off still ask whether the word is Vietnamese.
    for auto_correct in [true, false] {
        for (on, want) in [(true, true), (false, false)] {
            let mut engine = Engine::with_config(InputMethod::telex(), config(on, auto_correct));
            engine.process_str("jaa", Mode::Vietnamese);
            assert_eq!(engine.is_valid_word(), want, "on={on} auto_correct={auto_correct}");
        }
    }
}

#[test]
fn uppercase_follows_the_setting() {
    assert_eq!(typed(true, "JAAYJ"), "JẬY");
    assert_eq!(typed(true, "Jaayj"), "Jậy");
    assert_eq!(typed(false, "JAAYJ"), "JAAYJ");
    assert_eq!(typed(false, "Jaayj"), "Jaayj");
}

#[test]
fn without_auto_correct_the_setting_changes_nothing() {
    // Without auto-correct an invalid syllable still takes marks, so f, j and w do too.
    for keys in ["jaayj", "faayj", "waayj", "just", "jussive", "ferry", "vieetj"] {
        assert_eq!(typed_with(true, false, keys), typed_with(false, false, keys), "{keys}");
    }
    assert_eq!(typed_with(false, false, "jaayj"), "jậy");
}

#[test]
fn vni_follows_the_setting() {
    assert_eq!(typed_in(InputMethod::vni(), true, "ja65"), "jậ");
    assert_eq!(typed_in(InputMethod::vni(), true, "JA65"), "JẬ");
    assert_eq!(typed_in(InputMethod::vni(), false, "ja65"), "ja65");
    assert_eq!(typed_in(InputMethod::vni(), false, "fa61"), "fa61");
    for on in [true, false] {
        assert_eq!(typed_in(InputMethod::vni(), on, "za65"), "zậ");
        assert_eq!(typed_in(InputMethod::vni(), on, "vie65t"), "việt");
    }
}

#[test]
fn viqr_and_custom_follow_the_setting() {
    assert_eq!(typed_in(InputMethod::viqr(), true, "ja^."), "jậ");
    assert_eq!(typed_in(InputMethod::viqr(), false, "ja^."), "ja^.");
    assert_eq!(typed_in(InputMethod::viqr(), false, "za^."), "zậ");

    let custom = || InputMethod::from_definition("Custom", [("q", "DauSac"), ("a", "A_Â")]);
    assert_eq!(typed_in(custom(), true, "jaaq"), "jấ");
    assert_eq!(typed_in(custom(), false, "jaaq"), "jaaq");
    assert_eq!(typed_in(custom(), false, "baaq"), "bấ");
}

#[test]
fn backspace_follows_the_setting() {
    assert_eq!(typed(true, "jaayj<"), "jậ");
    assert_eq!(typed(true, "jaayj<y"), "jậy");
    assert_eq!(typed(false, "jaayj<"), "jaay");
    assert_eq!(typed(false, "zaayj<"), "zậ");
}

#[test]
fn raw_keys_survive_either_way() {
    for on in [true, false] {
        let mut engine = new_engine(InputMethod::telex(), on);
        engine.process_str("jaayj", Mode::Vietnamese);
        assert_eq!(engine.get_processed_str(OutputOptions::RAW), "jaayj");
        assert_eq!(typed(on, "jaayj!"), "jaayj");
    }
}

#[test]
fn a_change_applies_from_the_next_key() {
    assert_eq!(typed(false, "+jaayj"), "jậy");
    assert_eq!(typed(true, "-jaayj"), "jaayj");
    // The tone key comes after the change, so the new setting decides it.
    assert_eq!(typed(true, "jaay-j"), "jaayj");
    // Keys typed while off stay raw, so turning it on later has nothing to mark.
    assert_eq!(typed(false, "jaay+j"), "jaayj");
    // A tone already placed stays.
    assert_eq!(typed(true, "jaayj-"), "jậy");
}

#[test]
fn a_change_mid_word_matches_typing_with_the_new_setting() {
    // After turning it off, the next key sees the word as invalid, as if typed with it off.
    assert_eq!(typed(true, "fofa-f"), typed(false, "fofaf"));
    assert_eq!(typed(true, "fofa-f"), "fofaf");
    assert_eq!(typed(true, "jajo-j"), "jajoj");
}

#[test]
fn a_change_after_english_bypass() {
    // The doubled `s` undid the tone before the change, so the word keeps one `s`.
    assert_eq!(typed(true, "juss-ive"), "jusive");
    assert_eq!(typed(true, "juss-ive!"), "jussive");
    // Off from the start, the word never took a tone.
    assert_eq!(typed(false, "juss+ive"), "jussive");
}

#[test]
fn engines_keep_their_own_setting() {
    let mut on = new_engine(InputMethod::telex(), true);
    let mut off = new_engine(InputMethod::telex(), false);
    for key in "jaayj".chars() {
        on.process_key(key, Mode::Vietnamese);
        off.process_key(key, Mode::Vietnamese);
    }
    assert_eq!(on.output(), "jậy");
    assert_eq!(off.output(), "jaayj");
}

#[cfg(feature = "parallel")]
#[test]
fn batch_uses_the_default() {
    let out = bamboo_core::parallel::process_batch(
        &["jaayj", "vieetj"],
        &InputMethod::telex(),
        Mode::Vietnamese,
    );
    assert_eq!(out, ["jậy", "việt"]);
}
