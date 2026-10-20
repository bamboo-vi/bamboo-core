use bamboo_core::{Config, Engine, InputMethod, InputMethodPreset, Mode, ToneRefresh, ToneStyle};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_fuzz_never_panics(keys in "\\PC{0,100}") {
        let mut engine = Engine::new(InputMethod::telex());
        for ch in keys.chars() {
            engine.process_key(ch, Mode::Vietnamese);
            let out_str = engine.output_str();
            let out_cow = engine.output();
            if out_str == out_cow.as_ref() {
                prop_assert_eq!(out_str, out_cow.as_ref());
            }
        }
    }

    #[test]
    fn test_vni_fuzz_never_panics(keys in "\\PC{0,100}") {
        let mut engine = Engine::new(InputMethod::vni());
        for ch in keys.chars() {
            engine.process_key(ch, Mode::Vietnamese);
            let _ = engine.output();
        }
    }

    #[test]
    fn test_process_key_vs_delta_equivalence(keys in "[a-zA-Z0-9 ]{1,60}") {
        let mut engine_direct = Engine::new(InputMethod::telex());
        let mut engine_delta = Engine::new(InputMethod::telex());

        for ch in keys.chars() {
            engine_direct.process_key(ch, Mode::Vietnamese);
            let _edit = engine_delta.process_key_edit(ch, Mode::Vietnamese);

            prop_assert_eq!(engine_direct.output(), engine_delta.output());
        }
    }

    #[test]
    fn test_preset_vs_constructor_equivalence(keys in "[a-zA-Z0-9 ]{1,60}") {
        let mut engine1 = Engine::new(InputMethod::telex());
        let mut engine2 = Engine::new(InputMethod::from_preset(InputMethodPreset::Telex));

        for ch in keys.chars() {
            engine1.process_key(ch, Mode::Vietnamese);
            engine2.process_key(ch, Mode::Vietnamese);
            prop_assert_eq!(engine1.output(), engine2.output());
        }
    }

    #[test]
    fn test_undo_cleanliness(keys in "[a-z]{1,12}") {
        let mut engine = Engine::new(InputMethod::telex());
        let count = keys.chars().count();
        for ch in keys.chars() {
            engine.process_key(ch, Mode::Vietnamese);
        }

        // Deleting characters up to typed count should never panic and should leave buffer clean
        for _ in 0..count {
            engine.undo_last_key_with(ToneRefresh::PreserveSnapshot);
        }
        prop_assert_eq!(engine.output_str(), "");
    }

    #[test]
    fn test_config_combinations_never_panic(
        keys in "[a-zA-Z0-9 ]{1,40}",
        auto_correct in any::<bool>(),
        std_tone_style in any::<bool>(),
        free_marking in any::<bool>()
    ) {
        let config = Config::builder()
            .auto_correct(auto_correct)
            .tone_style(if std_tone_style {
                ToneStyle::Modern
            } else {
                ToneStyle::Traditional
            })
            .free_tone_marking(free_marking)
            .build();

        let mut engine = Engine::with_config(InputMethod::telex(), config);
        for ch in keys.chars() {
            engine.process_key(ch, Mode::Vietnamese);
        }
        let _ = engine.output();
    }
}
