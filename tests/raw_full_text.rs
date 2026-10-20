//! `FULL_TEXT` combined with other output options covers committed words too,
//! like Go bamboo-core's `GetProcessedString(EnglishMode | FullText)`.

use bamboo_core::{Engine, InputMethod, LastWordForm, Mode, OutputOptions};
use std::borrow::Cow;

const RAW_FULL: OutputOptions = OutputOptions::RAW.union(OutputOptions::FULL_TEXT);

fn typed(keys: &str) -> Engine {
    let mut e = Engine::new(InputMethod::telex());
    e.process_str(keys, Mode::Vietnamese);
    e
}

#[test]
fn raw_full_text_returns_keys_of_committed_words() {
    let e = typed("tooi oo HIEEUR");
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "tôi ô HIỂU");
    assert_eq!(e.get_processed_str(RAW_FULL), "tooi oo HIEEUR");
    // Without FULL_TEXT, RAW still covers only the active word.
    assert_eq!(e.get_processed_str(OutputOptions::RAW), "HIEEUR");
}

#[test]
fn full_text_applies_case_and_mark_options_to_committed_words() {
    let e = typed("Ddaays laf HIEEUR");
    let full = OutputOptions::FULL_TEXT;
    assert_eq!(e.get_processed_str(full), "Đấy là HIỂU");
    assert_eq!(e.get_processed_str(full | OutputOptions::TONE_LESS), "Đây la HIÊU");
    assert_eq!(e.get_processed_str(full | OutputOptions::MARK_LESS), "Day la HIEU");
    assert_eq!(e.get_processed_str(full | OutputOptions::LOWER_CASE), "đấy là hiểu");
    assert_eq!(e.get_processed_str(RAW_FULL | OutputOptions::LOWER_CASE), "ddaays laf hieeur");
}

#[test]
fn full_text_ignores_punctuation_mode_like_go() {
    let e = typed("tooi, HIEEUR");
    assert_eq!(
        e.get_processed_str(OutputOptions::FULL_TEXT | OutputOptions::PUNCTUATION_MODE),
        e.get_processed_str(OutputOptions::FULL_TEXT)
    );
}

#[test]
fn raw_full_text_with_empty_active_borrows() {
    let mut e = typed("tooi ");
    assert!(matches!(e.get_processed_str_cow(RAW_FULL), Cow::Borrowed("tooi ")));
    assert!(matches!(e.get_processed_str_cow(OutputOptions::FULL_TEXT), Cow::Borrowed("tôi ")));
    e.reset();
    assert_eq!(e.get_processed_str(RAW_FULL), "");
}

#[test]
fn explicit_commit_and_english_mode_keep_raw_text() {
    let mut e = typed("vieetj");
    e.commit();
    e.process_str("hello ", Mode::English);
    e.process_str("nam", Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "việthello nam");
    assert_eq!(e.get_processed_str(RAW_FULL), "vieetjhello nam");
}

#[test]
fn edits_to_the_active_word_keep_raw_text_consistent() {
    let mut e = typed("tooi HIEEUR");
    e.remove_last_output_char();
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "tôi HIỂ");
    assert_eq!(e.get_processed_str(RAW_FULL), "tooi HIEER");

    e.restore_last_word_as(LastWordForm::RawKeystrokes);
    assert_eq!(e.get_processed_str(RAW_FULL), "tooi HIEER");

    e.undo_last_key();
    assert_eq!(
        e.get_processed_str(RAW_FULL),
        format!("tooi {}", e.get_processed_str(OutputOptions::RAW))
    );
}

#[test]
fn backspace_on_empty_active_leaves_committed_raw_text() {
    let mut e = typed("tooi ");
    e.remove_last_output_char();
    e.undo_last_key();
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "tôi ");
    assert_eq!(e.get_processed_str(RAW_FULL), "tooi ");
}
