// Expected outputs come from Go bamboo-core (RemoveLastChar) with the same keys and settings.
use bamboo_core::{Config, Engine, InputMethod, Mode, ToneRefresh, ToneStyle};

fn engine(free_tone: bool, modern: bool) -> Engine {
    let config = Config::builder()
        .free_tone_marking(free_tone)
        .tone_style(if modern { ToneStyle::Traditional } else { ToneStyle::Modern })
        .auto_correct(false)
        .build();
    Engine::with_config(InputMethod::telex(), config)
}

fn typed_then_deleted(free_tone: bool, modern: bool, keys: &str) -> String {
    let mut engine = engine(free_tone, modern);
    for key in keys.chars() {
        engine.process_key(key, Mode::Vietnamese);
    }
    engine.remove_last_output_char();
    engine.output().into_owned()
}

#[test]
fn deleting_from_an_invalid_word_keeps_the_tone_in_place() {
    assert_eq!(typed_then_deleted(true, true, "craxyuk"), "crãyu");
    assert_eq!(typed_then_deleted(true, true, "afuvw"), "àuv");
}

#[test]
fn deleting_without_free_tone_marking_keeps_the_tone_in_place() {
    assert_eq!(typed_then_deleted(false, false, "hoafn"), "hoà");
}

#[test]
fn deleting_from_a_valid_word_moves_the_tone() {
    assert_eq!(typed_then_deleted(true, false, "hoafn"), "hòa");
    assert_eq!(typed_then_deleted(true, true, "hoanf"), "hoà");
}

#[test]
fn undoing_a_key_keeps_the_tone_for_the_next_key() {
    let mut engine = engine(true, true);
    for key in "craxyuk".chars() {
        engine.process_key(key, Mode::Vietnamese);
    }
    engine.undo_last_key_with(ToneRefresh::Recompute);
    engine.process_key('n', Mode::Vietnamese);
    assert_eq!(engine.output(), "crãyun");
}
