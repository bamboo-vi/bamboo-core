//! Issue #41: composition overflow must finish the composition as a
//! **signaled** word boundary and start a new composition with the current
//! key, instead of silently resetting the composition.
//!
//! A silent reset made delta-based hosts delete dozens of already-committed
//! characters at once (`delete_chars: 31, insert: ""` at key 33 of a 40-key
//! run), which they rendered as a mass deletion on screen.

use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};

const VN: Mode = Mode::Vietnamese;

fn is_break_key(key: char) -> bool {
    key == ' ' || key.is_ascii_punctuation() || key.is_ascii_digit()
}

/// Simulates a delta-based host as described in the README: `app` is the text
/// already committed to the application, `preedit` mirrors `output_str()`.
///
/// Applies every `EditDelta` and asserts the host-side invariants:
/// * a signaled boundary (`EditDelta::committed`) never deletes text,
/// * the preedit mirror stays in sync with `engine.output_str()`,
/// * `delete_bytes` matches the deleted suffix exactly (UTF-8 accounting).
struct HostSim {
    app: String,
    preedit: String,
    /// 1-based indices of the keys that signaled a forced word boundary.
    boundary_keys: Vec<usize>,
}

impl HostSim {
    fn new() -> Self {
        Self { app: String::new(), preedit: String::new(), boundary_keys: Vec::new() }
    }

    fn on_key(&mut self, engine: &mut Engine, key_index: usize, key: char) {
        let old_preedit = self.preedit.clone();
        let edit = engine.process_key_edit(key, VN);
        let (delete_chars, delete_bytes, insert) =
            (edit.delete_chars, edit.delete_bytes, edit.insert.to_string());

        if edit.committed {
            // Contract: the engine committed the whole previous preedit as a
            // finished word and started a new composition with `key`. Flush
            // the mirror to the application before applying the edit, which
            // is relative to the now-empty preedit.
            self.app.push_str(&old_preedit);
            self.preedit.clear();
            self.boundary_keys.push(key_index);
            assert_eq!(delete_chars, 0, "key {key_index}: signaled boundary must not delete");
            assert_eq!(delete_bytes, 0, "key {key_index}: signaled boundary must not delete");
            assert_eq!(
                insert,
                engine.output_str(),
                "key {key_index}: boundary insert must be the whole new preedit"
            );
        }

        let keep = self.preedit.chars().count().saturating_sub(delete_chars);
        let kept: String = self.preedit.chars().take(keep).collect();
        let deleted: String = self.preedit.chars().skip(keep).collect();
        assert_eq!(
            delete_bytes,
            deleted.len(),
            "key {key_index}: delete_bytes must match the deleted suffix"
        );
        self.preedit = kept;
        self.preedit.push_str(&insert);

        if is_break_key(key) {
            // Legacy contract (unchanged by the fix): the host recognizes the
            // break key and commits its mirror plus the key itself; the delta
            // only syncs the preedit to empty.
            self.app.push_str(&old_preedit);
            self.app.push(key);
            self.preedit.clear();
        }

        assert_eq!(
            self.preedit,
            engine.output_str(),
            "key {key_index}: host preedit mirror out of sync"
        );
    }

    /// Models `undo_last_key` for a host: the engine has no delta API for
    /// undo, so the host resyncs its mirror from `output_str()`.
    fn on_undo(&mut self, engine: &mut Engine) {
        engine.undo_last_key();
        self.preedit.clear();
        self.preedit.push_str(engine.output_str());
    }

    fn visible(&self) -> String {
        format!("{}{}", self.app, self.preedit)
    }
}

/// Feeds `keys` through a fresh engine with per-key invariants (callers use
/// unbroken runs without word-break keys):
/// * `sim.visible()` (what the host shows) equals `FULL_TEXT` (engine truth),
/// * `RAW | FULL_TEXT` equals every key typed so far (no slot ever dropped),
/// * `output_str()` shrinks only at a signaled boundary.
fn run(engine: &mut Engine, keys: &str) -> HostSim {
    let mut sim = HostSim::new();
    let mut typed = String::new();
    for (i, key) in keys.chars().enumerate() {
        let key_index = i + 1;
        let out_before = engine.output_str().chars().count();
        sim.on_key(engine, key_index, key);
        typed.push(key);

        assert_eq!(
            sim.visible(),
            engine.get_processed_str(OutputOptions::FULL_TEXT),
            "key {key_index}: host lost text (visible != FULL_TEXT)"
        );
        assert_eq!(
            engine.get_processed_str(OutputOptions::RAW | OutputOptions::FULL_TEXT),
            typed,
            "key {key_index}: a typed key was lost"
        );
        if sim.boundary_keys.last() != Some(&key_index) {
            let out_len = engine.output_str().chars().count();
            assert!(
                out_len >= out_before,
                "key {key_index}: output_str shrank {out_before} -> {out_len} without a signaled boundary"
            );
        }
    }
    sim
}

/// Same as [`run`] but without the `FULL_TEXT` check (tracking disabled).
fn run_without_full_text(engine: &mut Engine, keys: &str) -> HostSim {
    let mut sim = HostSim::new();
    for (i, key) in keys.chars().enumerate() {
        let key_index = i + 1;
        let out_before = engine.output_str().chars().count();
        sim.on_key(engine, key_index, key);
        if sim.boundary_keys.last() != Some(&key_index) {
            let out_len = engine.output_str().chars().count();
            assert!(
                out_len >= out_before,
                "key {key_index}: output_str shrank {out_before} -> {out_len} without a signaled boundary"
            );
        }
    }
    sim
}

#[test]
fn letter_run_finishes_word_at_slot_33() {
    let mut engine = Engine::new(InputMethod::telex());
    let sim = run(&mut engine, &"b".repeat(40));
    assert_eq!(sim.boundary_keys, vec![33], "one slot per letter: boundary at key 33");
    assert_eq!(sim.visible(), "b".repeat(40));
}

#[test]
fn issue_repro_o_run_finishes_word_at_slot_33() {
    let mut engine = Engine::new(InputMethod::telex());
    let sim = run(&mut engine, &"o".repeat(40));
    assert_eq!(sim.boundary_keys, vec![33], "issue #41 repro: boundary at key 33");
    assert_eq!(engine.output_str().chars().count(), 8);
    assert_eq!(sim.visible(), "o".repeat(40));
}

#[test]
fn slot_heavy_runs_count_slots_not_raw_keys() {
    // "aa" -> "â" spends an extra transformation slot, so these runs hit the
    // 32-slot budget before 32 raw keys (issue: "words with many marks must
    // collapse by slots, not by raw key count").
    for keys in ["a".repeat(40), "aas".repeat(14)] {
        let mut engine = Engine::new(InputMethod::telex());
        let sim = run(&mut engine, &keys);
        assert_eq!(
            sim.boundary_keys.len(),
            1,
            "{keys:?}: exactly one boundary in {} keys",
            keys.chars().count()
        );
        assert!(
            sim.boundary_keys[0] < 33,
            "{keys:?}: slot-heavy stream must finish before key 33, got {}",
            sim.boundary_keys[0]
        );
    }
}

#[test]
fn multi_slot_key_at_full_buffer_starts_new_word() {
    // 28 * 'b' + "luo" = 31 slots; 'w' on the "uơ/ưo"-like tail needs two
    // slots (mark + horn spread), i.e. 33 > 32: the word must be finished and
    // 'w' must start a new composition without any slot being dropped.
    let mut engine = Engine::new(InputMethod::telex());
    let keys = format!("{}luow", "b".repeat(28));
    let sim = run(&mut engine, &keys);
    assert_eq!(sim.boundary_keys, vec![32], "the two-slot 'w' triggers the boundary");
    assert_eq!(engine.output_str(), "w");
    assert_eq!(engine.active_len(), 1);
    assert_eq!(sim.visible(), format!("{}luo", "b".repeat(28)) + "w");
}

#[test]
fn unicode_run_never_loses_text() {
    let mut engine = Engine::new(InputMethod::telex());
    let sim = run(&mut engine, &"ậ".repeat(40));
    assert_eq!(sim.boundary_keys, vec![33]);
    assert_eq!(sim.visible(), "ậ".repeat(40));
}

#[test]
fn word_break_delta_contract_is_unchanged() {
    let mut engine = Engine::new(InputMethod::telex());
    let _ = engine.process_key_edit('b', VN);
    let _ = engine.process_key_edit('b', VN);
    let edit = engine.process_key_edit(' ', VN);
    assert!(!edit.committed, "word break must keep the legacy delta shape");
    assert_eq!(edit.delete_chars, 2);
    assert_eq!(edit.delete_bytes, 2);
    assert_eq!(edit.insert, "");
    assert_eq!(engine.output_str(), "");
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "bb ");
}

#[test]
fn backspace_after_boundary_keeps_committed_word() {
    let mut engine = Engine::new(InputMethod::telex());
    let mut sim = run(&mut engine, &"b".repeat(35));
    assert_eq!(sim.boundary_keys, vec![33]);
    assert_eq!(sim.visible(), "b".repeat(35));

    // Keys 33..35 restarted the composition after the boundary, so its
    // preedit is "bbb" and undo only walks back within that word.
    sim.on_undo(&mut engine);
    sim.on_undo(&mut engine);
    assert_eq!(engine.output_str(), "b");
    assert_eq!(sim.visible(), "b".repeat(33), "undo must never touch the committed word");
    assert_eq!(sim.visible(), engine.get_processed_str(OutputOptions::FULL_TEXT));

    sim.on_undo(&mut engine);
    assert_eq!(engine.output_str(), "");
    assert_eq!(sim.visible(), "b".repeat(32), "undo must stop at the boundary");

    sim.on_undo(&mut engine);
    assert_eq!(sim.visible(), "b".repeat(32), "undo past the boundary must be a no-op");

    sim.on_key(&mut engine, 36, 'x');
    assert_eq!(engine.output_str(), "x");
    assert_eq!(sim.visible(), format!("{}x", "b".repeat(32)));
    assert_eq!(sim.visible(), engine.get_processed_str(OutputOptions::FULL_TEXT));
}

#[test]
fn boundary_is_signaled_without_full_text_tracking() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.set_full_text_tracking(false);
    let sim = run_without_full_text(&mut engine, &"b".repeat(40));
    assert_eq!(sim.boundary_keys, vec![33]);
    assert_eq!(sim.visible(), "b".repeat(40));
}

#[test]
fn process_key_hosts_see_the_boundary_signal() {
    let mut engine = Engine::new(InputMethod::telex());
    for i in 0..32 {
        engine.process_key('b', VN);
        assert!(!engine.just_committed(), "key {}: no boundary yet", i + 1);
    }
    engine.process_key('b', VN);
    assert!(engine.just_committed(), "key 33 finishes the word");
    assert_eq!(engine.output_str(), "b");
}

#[test]
fn exact_fit_two_slot_key_does_not_split_the_word() {
    // 28 * 'b' + "luo" would overflow (see multi_slot_key test), but 27 * 'b'
    // + "luo" = 30 slots leaves room for the two-slot 'w': 32 slots exactly,
    // so the word must NOT be split and no slot may be lost.
    // auto_correct off: it would restore raw keys for the invalid long word
    // ("…bluow"), which is a different code path.
    let config = bamboo_core::Config { auto_correct: false, ..Default::default() };
    let mut engine = Engine::with_config(InputMethod::telex(), config);
    let keys = format!("{}luow", "b".repeat(27));
    let sim = run(&mut engine, &keys);
    assert!(sim.boundary_keys.is_empty(), "exact fit must not split the word");
    assert_eq!(engine.output_str(), format!("{}lươ", "b".repeat(27)));
    assert_eq!(engine.active_len(), 32);
    assert_eq!(sim.visible(), format!("{}lươ", "b".repeat(27)));
}

#[test]
fn undo_cycle_runs_stay_consistent_through_boundary() {
    // "ass" -> "as": the second 's' undoes the tone, so the stream mixes
    // literal, tone and undo-cycle keys while it crosses the slot budget.
    let mut engine = Engine::new(InputMethod::telex());
    let sim = run(&mut engine, &"ass".repeat(16));
    assert!(!sim.boundary_keys.is_empty(), "48 slot-heavy keys must cross the budget");
    assert_eq!(sim.visible(), engine.get_processed_str(OutputOptions::FULL_TEXT));
}

#[test]
fn delta_resyncs_after_host_mutations() {
    use bamboo_core::LastWordForm;

    // Host-initiated commit.
    let mut engine = Engine::new(InputMethod::telex());
    let _ = engine.process_key_edit('b', VN);
    let _ = engine.process_key_edit('b', VN);
    engine.commit();
    let edit = engine.process_key_edit('x', VN);
    assert_eq!(edit.delete_chars, 0, "delta after commit must diff the empty preedit");
    assert_eq!(edit.insert, "x");

    // Undo the last keystroke.
    let mut engine = Engine::new(InputMethod::telex());
    let _ = engine.process_key_edit('b', VN);
    let _ = engine.process_key_edit('b', VN);
    engine.undo_last_key();
    let edit = engine.process_key_edit('x', VN);
    assert_eq!(edit.delete_chars, 0, "delta after undo must diff the undone preedit");
    assert_eq!(edit.insert, "x");
    assert_eq!(engine.output_str(), "bx");

    // Remove the last output character.
    let mut engine = Engine::new(InputMethod::telex());
    let _ = engine.process_key_edit('b', VN);
    let _ = engine.process_key_edit('b', VN);
    engine.remove_last_output_char();
    let edit = engine.process_key_edit('x', VN);
    assert_eq!(edit.delete_chars, 0, "delta after remove_last_output_char must resync");
    assert_eq!(edit.insert, "x");

    // Restore the last word as raw keystrokes.
    let mut engine = Engine::new(InputMethod::telex());
    let _ = engine.process_key_edit('w', VN);
    let _ = engine.process_key_edit('a', VN);
    let _ = engine.process_key_edit('a', VN);
    engine.restore_last_word_as(LastWordForm::RawKeystrokes);
    let edit = engine.process_key_edit('x', VN);
    assert_eq!(edit.delete_chars, 0, "delta after restore must resync");
    assert_eq!(edit.insert, "x");

    // Rebuild from existing text.
    let mut engine = Engine::new(InputMethod::telex());
    engine.rebuild_from_text("bb");
    let edit = engine.process_key_edit('x', VN);
    assert_eq!(edit.delete_chars, 0, "delta after rebuild must resync");
    assert_eq!(edit.insert, "x");
}

#[test]
fn english_mode_overflow_is_signaled() {
    let mut engine = Engine::new(InputMethod::telex());
    let mut boundary_keys = Vec::new();
    let mut typed = String::new();
    for (i, key) in "b".repeat(40).chars().enumerate() {
        let edit = engine.process_key_edit(key, Mode::English);
        typed.push(key);
        if edit.committed {
            boundary_keys.push(i + 1);
            assert_eq!(edit.delete_chars, 0);
        }
        assert_eq!(
            engine.get_processed_str(OutputOptions::FULL_TEXT),
            typed,
            "key {}: English-mode raw run must lose no key",
            i + 1
        );
    }
    assert_eq!(boundary_keys, vec![33]);
}
