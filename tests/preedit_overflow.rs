//! Overflow behavior past the active-transformation window: keys must never be
//! dropped and the preedit must not truncate mid-word (see rust-gaps.md case 2).

use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};

fn type_str(keys: &str, mode: Mode) -> Engine {
    let mut e = Engine::new(InputMethod::telex());
    for k in keys.chars() {
        e.process_key(k, mode);
    }
    e
}

#[test]
fn english_identifier_23_chars_stays_in_preedit() {
    let e = type_str("getUserNameByIdentifier", Mode::English);
    assert_eq!(e.output_str(), "getUserNameByIdentifier");
    assert_eq!(e.committed_text_len(), 0);
}

#[test]
fn english_identifier_39_chars_loses_nothing() {
    let keys = "very_long_variable_name_that_keeps_going";
    let e = type_str(keys, Mode::English);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), keys);
}

#[test]
fn vietnamese_raw_run_past_window_loses_no_keys() {
    let keys = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let e = type_str(keys, Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), keys);
}

#[test]
fn backspace_after_overflow_cannot_cross_silent_commit() {
    let keys = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let mut e = type_str(keys, Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), keys);
    e.remove_last_char(true);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), &keys[..39]);
    // Undo drains the post-overflow keys but must stop at the silent commit:
    // committed text is not backspace-undoable (contract of `commit()`).
    for _ in 0..keys.len() {
        e.remove_last_char(true);
    }
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), &keys[..32]);
}

#[test]
fn preedit_cache_matches_flatten_after_raw_append_and_edits() {
    // The raw-append path maintains `cached_output` incrementally; it must
    // agree with a full flatten on every entry point.
    let mut e = type_str("getUserNameByIdentifier", Mode::English);
    assert_eq!(e.output_str(), e.get_processed_str(OutputOptions::FULL_TEXT));
    e.remove_last_output_char();
    assert_eq!(e.output_str(), e.get_processed_str(OutputOptions::FULL_TEXT));

    let mut v = type_str("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", Mode::Vietnamese);
    v.remove_last_char(true);
    let full = v.get_processed_str(OutputOptions::FULL_TEXT);
    assert_eq!(full, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    // Preedit = the uncommitted tail after the 32-char silent commit.
    assert_eq!(v.output_str(), &full[32..]);
}
