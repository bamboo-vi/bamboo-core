use bamboo_core::{Config, Engine, InputMethod, LastWordForm, Mode, ToneRefresh, ToneStyle};

fn typed(keys: &str) -> Engine {
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str(keys, Mode::Vietnamese);
    engine
}

#[test]
#[allow(deprecated)]
fn legacy_backspace_matches_named_policies() {
    let mut legacy_recompute = typed("tieengs");
    let mut named_recompute = typed("tieengs");
    legacy_recompute.remove_last_char(true);
    named_recompute.undo_last_key();
    assert_eq!(legacy_recompute.output(), named_recompute.output());

    let mut legacy_preserve = typed("tieengs");
    let mut named_preserve = typed("tieengs");
    legacy_preserve.remove_last_char(false);
    named_preserve.undo_last_key_with(ToneRefresh::PreserveSnapshot);
    assert_eq!(legacy_preserve.output(), named_preserve.output());
}

#[test]
#[allow(deprecated)]
fn legacy_restore_matches_named_word_forms() {
    let mut legacy_raw = typed("tieengs");
    let mut named_raw = typed("tieengs");
    legacy_raw.restore_last_word(false);
    named_raw.restore_last_word_as(LastWordForm::RawKeystrokes);
    assert_eq!(legacy_raw.output(), named_raw.output());

    legacy_raw.restore_last_word(true);
    named_raw.restore_last_word_as(LastWordForm::Vietnamese);
    assert_eq!(legacy_raw.output(), named_raw.output());
}

#[test]
#[allow(deprecated)]
fn legacy_validation_matches_named_queries() {
    let engine = typed("tooi");
    assert_eq!(engine.is_valid(true), engine.is_valid_word());
    assert_eq!(engine.is_valid(false), engine.is_valid_prefix());
}

#[test]
#[allow(deprecated)]
fn legacy_delta_matches_named_edit() {
    let mut legacy = Engine::new(InputMethod::telex());
    let mut named = Engine::new(InputMethod::telex());

    for key in "as ".chars() {
        let (delete_chars, delete_bytes, insert) = legacy.process_key_delta(key, Mode::Vietnamese);
        let edit = named.process_key_edit(key, Mode::Vietnamese);
        assert_eq!(delete_chars, edit.delete_chars);
        assert_eq!(delete_bytes, edit.delete_bytes);
        assert_eq!(insert, edit.insert);
    }

    let mut legacy_insert = String::new();
    let mut named_insert = String::new();
    let legacy_delete = legacy.process_key_delta_into('s', Mode::Vietnamese, &mut legacy_insert);
    let named_delete = named.process_key_edit_into('s', Mode::Vietnamese, &mut named_insert);
    assert_eq!(legacy_delete, named_delete);
    assert_eq!(legacy_insert, named_insert);
}

#[test]
#[allow(deprecated)]
fn legacy_tone_style_builder_matches_enum() {
    let modern_legacy = Config::builder().std_tone_style(true).build();
    let modern_named = Config::builder().tone_style(ToneStyle::Modern).build();
    assert_eq!(modern_legacy, modern_named);
    assert_eq!(modern_named.tone_style(), ToneStyle::Modern);

    let traditional_legacy = Config::builder().std_tone_style(false).build();
    let traditional_named = Config::builder().tone_style(ToneStyle::Traditional).build();
    assert_eq!(traditional_legacy, traditional_named);
    assert_eq!(traditional_named.tone_style(), ToneStyle::Traditional);
}
