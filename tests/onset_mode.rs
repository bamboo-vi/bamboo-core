use bamboo_core::{Config, Engine, InputMethod, Mode, OnsetMode};

fn config(mode: OnsetMode) -> Config {
    Config::builder().onset_mode(mode).build()
}

/// Types `keys` in Telex: `<` deletes the last character, and `1`, `2`, `3` switch to
/// Standard, Z and Extended.
fn typed(mode: OnsetMode, keys: &str) -> String {
    let mut engine = Engine::with_config(InputMethod::telex(), config(mode));
    for key in keys.chars() {
        match key {
            '<' => engine.remove_last_output_char(),
            '1' => engine.set_config(config(OnsetMode::Standard)),
            '2' => engine.set_config(config(OnsetMode::Z)),
            '3' => engine.set_config(config(OnsetMode::Extended)),
            _ => engine.process_key(key, Mode::Vietnamese),
        }
    }
    engine.output().into_owned()
}

#[test]
fn z_is_the_default() {
    assert_eq!(Config::default().onset_mode, OnsetMode::Z);
    assert_eq!(Engine::new(InputMethod::telex()).config().onset_mode, OnsetMode::Z);
}

#[test]
fn standard_takes_no_extra_onset() {
    for keys in ["zaayj", "faayj", "jaayj", "waayj"] {
        assert_eq!(typed(OnsetMode::Standard, keys), keys);
    }
    assert_eq!(typed(OnsetMode::Standard, "vieetj"), "việt");
}

#[test]
fn z_takes_z_only() {
    assert_eq!(typed(OnsetMode::Z, "zaayj"), "zậy");
    for keys in ["faayj", "jaayj", "waayj"] {
        assert_eq!(typed(OnsetMode::Z, keys), keys);
    }
}

#[test]
fn extended_takes_z_f_j_and_w() {
    assert_eq!(typed(OnsetMode::Extended, "zaayj"), "zậy");
    assert_eq!(typed(OnsetMode::Extended, "faayj"), "fậy");
    assert_eq!(typed(OnsetMode::Extended, "jaayj"), "jậy");
    assert_eq!(typed(OnsetMode::Extended, "waayj"), "wậy");
}

#[test]
fn english_words_change_only_in_extended() {
    for word in ["just", "jussive", "ferry"] {
        assert_eq!(typed(OnsetMode::Standard, word), word);
        assert_eq!(typed(OnsetMode::Z, word), word);
    }
    assert_eq!(typed(OnsetMode::Extended, "just"), "jút");
    assert_eq!(typed(OnsetMode::Extended, "jussive"), "jusive");
    assert_eq!(typed(OnsetMode::Extended, "ferry"), "fery");
}

#[test]
fn backspace_follows_the_mode() {
    assert_eq!(typed(OnsetMode::Standard, "zaayj<"), "zaay");
    assert_eq!(typed(OnsetMode::Z, "zaayj<"), "zậ");
    assert_eq!(typed(OnsetMode::Z, "zaayj<y"), "zậy");
    assert_eq!(typed(OnsetMode::Z, "jaayj<"), "jaay");
    assert_eq!(typed(OnsetMode::Extended, "jaayj<"), "jậ");
    assert_eq!(typed(OnsetMode::Extended, "jaayj<y"), "jậy");
}

#[test]
fn a_new_mode_applies_from_the_next_key() {
    assert_eq!(typed(OnsetMode::Z, "3jaayj"), "jậy");
    assert_eq!(typed(OnsetMode::Extended, "2jaayj"), "jaayj");
    assert_eq!(typed(OnsetMode::Z, "1zaayj"), "zaayj");
    // The tone key comes after the switch, so the new mode decides it.
    assert_eq!(typed(OnsetMode::Extended, "jaay2j"), "jaayj");
    // Switching alone leaves the word as it is.
    assert_eq!(typed(OnsetMode::Extended, "jaayj2"), "jậy");
    // The next key reads the word again with the new mode, as a fresh engine would.
    assert_eq!(typed(OnsetMode::Z, "zajo1z"), "zao");
}
