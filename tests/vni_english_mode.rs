//! VNI tone/mark keys are digits, which also count as word breaks. Once an IME
//! wrapper switches an invalid word to `Mode::English`, those digits must stay
//! in the word (as in the Go bamboo-core) so restore, raw output and backspace
//! still see the whole word.

use bamboo_core::{Config, Engine, InputMethod, LastWordForm, Mode, OutputOptions};

/// Types `keys` the way an IME wrapper does: Vietnamese until the word stops
/// being valid, English afterwards. `<` deletes the last output character.
fn type_like_wrapper(engine: &mut Engine, keys: &str) {
    for k in keys.chars() {
        if k == '<' {
            engine.remove_last_output_char();
            continue;
        }
        let vn = engine.get_processed_str(OutputOptions::LOWER_CASE);
        let mode = if !vn.is_empty() && !engine.is_valid_prefix() {
            Mode::English
        } else {
            Mode::Vietnamese
        };
        engine.process_key(k, mode);
    }
}

// Auto-correct would restore invalid words itself; the wrapper does that instead.
fn new_engine(method: InputMethod) -> Engine {
    Engine::with_config(method, Config::builder().auto_correct(false).build())
}

fn full_text(engine: &Engine) -> String {
    engine.get_processed_str(OutputOptions::FULL_TEXT)
}

fn raw(engine: &Engine) -> String {
    engine.get_processed_str(OutputOptions::RAW)
}

#[test]
fn vni_digit_in_english_mode_stays_in_word_for_restore() {
    let mut e = new_engine(InputMethod::vni());
    type_like_wrapper(&mut e, "e6ng2");
    assert_eq!(full_text(&e), "êng2");
    assert_eq!(raw(&e), "e6ng2");
    e.restore_last_word_as(LastWordForm::RawKeystrokes);
    assert_eq!(full_text(&e), "e6ng2");
}

#[test]
fn vni_raw_keeps_whole_word_across_digits() {
    let mut e = new_engine(InputMethod::vni());
    type_like_wrapper(&mut e, "xre6po6c");
    assert_eq!(raw(&e), "xre6po6c");
    assert_eq!(full_text(&e), "xre6po6c");
}

#[test]
fn vni_backspace_reaches_digit_typed_in_english_mode() {
    let mut e = new_engine(InputMethod::vni());
    type_like_wrapper(&mut e, "ing3<");
    assert_eq!(full_text(&e), "ing");
    type_like_wrapper(&mut e, "3");
    assert_eq!(full_text(&e), "ing3");
}

#[test]
fn vni_space_in_english_mode_still_ends_word() {
    let mut e = new_engine(InputMethod::vni());
    type_like_wrapper(&mut e, "xre6 ");
    assert_eq!(raw(&e), "");
    assert_eq!(full_text(&e), "xre6 ");
}

#[test]
fn telex_digit_in_english_mode_still_ends_word() {
    let mut e = new_engine(InputMethod::telex());
    type_like_wrapper(&mut e, "xra1");
    assert_eq!(raw(&e), "");
    assert_eq!(full_text(&e), "xra1");
}
