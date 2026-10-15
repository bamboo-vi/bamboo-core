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
