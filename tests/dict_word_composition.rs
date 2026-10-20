//! Dict words whose onsets break the Vietnamese spelling rules (`crông`, `kpăng`,
//! `đrăk`, `kilôgam`) must still compose when typed with Telex keys.
//!
//! The dictionary shipped with Ngó Sen lists them; this checks the core side of
//! that round trip, with the same config the Ngó Sen addon applies by default.

use bamboo_core::{Config, Engine, InputMethod, Mode, ToneStyle, W2uMode};

fn type_telex(keys: &str) -> String {
    let config = Config::builder()
        .free_tone_marking(true)
        .tone_style(ToneStyle::Traditional)
        .auto_correct(false)
        .w2u_mode(W2uMode::NonStart)
        .build();
    let mut e = Engine::with_config(InputMethod::telex(), config);
    e.process_str(keys, Mode::Vietnamese);
    e.output().into_owned()
}

#[test]
fn cluster_onset_dict_words_compose() {
    for (keys, want) in
        [("croong", "crông"), ("kpawng", "kpăng"), ("ddrawk", "đrăk"), ("kiloogam", "kilôgam")]
    {
        assert_eq!(type_telex(keys), want, "typing {keys:?}");
    }
}

/// The core's own default is `auto_correct: true`, which restores these words too.
/// Ngó Sen turns it off (`EngineSetOption` hardcodes `auto_correct(false)`), and then
/// the core composes every word — that is the config the report refers to.
#[test]
fn auto_correct_makes_the_core_restore_them_too() {
    for (keys, want) in
        [("croong", "crông"), ("kpawng", "kpăng"), ("ddrawk", "đrăk"), ("kiloo", "kilô")]
    {
        let mut e = Engine::new(InputMethod::telex());
        e.process_str(keys, Mode::Vietnamese);
        assert_ne!(e.output().into_owned(), want, "typing {keys:?}");

        let config = Config::builder().auto_correct(false).build();
        let mut e = Engine::with_config(InputMethod::telex(), config);
        e.process_str(keys, Mode::Vietnamese);
        assert_eq!(e.output().into_owned(), want, "typing {keys:?} (auto_correct off)");
    }
}
