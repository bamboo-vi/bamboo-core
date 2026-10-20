//! `w` typed where it cannot mark a vowel becomes `ư`, depending on `W2uMode`.
//! Expected strings come from the Go bamboo-core used by fcitx5-lotus (`SetW2UMode`).

use bamboo_core::{Config, Engine, InputMethod, Mode, ToneStyle, W2uMode};

// auto_correct would turn syllables such as `wơ` back into raw keys; Go leaves that to the caller.
fn typed(im: InputMethod, mode: W2uMode, keys: &str) -> String {
    let config = Config::builder().w2u_mode(mode).auto_correct(false).build();
    let mut engine = Engine::with_config(im, config);
    engine.process_str(keys, Mode::Vietnamese);
    engine.output().into_owned()
}

fn check(im: &InputMethod, mode: W2uMode, cases: &[(&str, &str)]) {
    for &(keys, expected) in cases {
        assert_eq!(typed(im.clone(), mode, keys), expected, "{mode:?} keys {keys:?}");
    }
}

#[test]
fn default_is_disabled() {
    assert_eq!(Config::default().w2u_mode, W2uMode::Disabled);
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str("nhw", Mode::Vietnamese);
    assert_eq!(engine.output(), "nhw");
}

// Words starting with `w` such as `wow` are left out: unlike Go, this crate puts no horn on `wo`.
#[test]
fn telex_disabled() {
    check(
        &InputMethod::telex(),
        W2uMode::Disabled,
        &[
            ("w", "w"),
            ("nhw", "nhw"),
            ("tw", "tw"),
            ("uw", "ư"),
            ("W", "W"),
            ("NHW", "NHW"),
            ("ww", "ww"),
            ("uww", "uw"),
        ],
    );
}

#[test]
fn telex_non_start() {
    check(
        &InputMethod::telex(),
        W2uMode::NonStart,
        &[
            ("w", "w"),
            ("nhw", "như"),
            ("wa", "wa"),
            ("nwowcs", "nước"),
            ("uw", "ư"),
            ("W", "W"),
            ("NHW", "NHƯ"),
            ("Wa", "Wa"),
            ("tw", "tư"),
            ("nhww", "nhw"),
            ("uww", "uw"),
            ("huwowngs", "hướng"),
            ("w w", "w"),
        ],
    );
}

#[test]
fn telex_everywhere() {
    check(
        &InputMethod::telex(),
        W2uMode::Everywhere,
        &[
            ("w", "ư"),
            ("nhw", "như"),
            ("wow", "ươ"),
            ("nwowcs", "nước"),
            ("uw", "ư"),
            ("W", "Ư"),
            ("NHW", "NHƯ"),
            ("Wow", "Ươ"),
            ("ww", "w"),
            ("nhww", "nhw"),
            ("uww", "uw"),
            ("w w", "ư"),
        ],
    );
}

// Go applies the mode to any method where `w` is a plain letter, so VNI changes too
// once the mode is on; with the default it types as before.
#[test]
fn vni() {
    let vni = InputMethod::vni();
    check(&vni, W2uMode::Disabled, &[("w", "w"), ("nhw", "nhw"), ("uw", "uw")]);
    check(&vni, W2uMode::NonStart, &[("w", "w"), ("nhw", "như"), ("u7", "ư")]);
    check(&vni, W2uMode::Everywhere, &[("w", "ư"), ("W", "Ư"), ("u7", "ư")]);
}

#[test]
fn mode_survives_flags() {
    for mode in [W2uMode::Disabled, W2uMode::NonStart, W2uMode::Everywhere] {
        let config = Config::builder().w2u_mode(mode).tone_style(ToneStyle::Traditional).build();
        assert_eq!(Config::from_flags(config.to_flags()), config);
    }
}

#[test]
fn set_config_changes_mode() {
    let mut engine = Engine::with_config(
        InputMethod::telex(),
        Config::builder().w2u_mode(W2uMode::NonStart).build(),
    );
    engine.process_str("nhw", Mode::Vietnamese);
    assert_eq!(engine.output(), "như");
    engine.reset();
    engine.set_config(Config::default());
    engine.process_str("nhw", Mode::Vietnamese);
    assert_eq!(engine.output(), "nhw");
}
