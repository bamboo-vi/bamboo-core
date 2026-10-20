use bamboo_core::{Engine, InputMethod, Mode};

fn verify_keystrokes(im: InputMethod, sequence: &str, expected_steps: &[&str]) {
    let mut engine = Engine::new(im);
    let steps: Vec<char> = sequence.chars().collect();

    assert_eq!(steps.len(), expected_steps.len(), "Sequence and expected steps length mismatch");

    for (i, &ch) in steps.iter().enumerate() {
        engine.process_key(ch, Mode::Vietnamese);
        let output = engine.output();
        assert_eq!(
            output, expected_steps[i],
            "Failed at step {}: typed '{}', expected '{}', got '{}'",
            i, ch, expected_steps[i], output
        );
    }
}

#[test]
fn test_telex_muoons_keystrokes() {
    let im = InputMethod::telex();
    verify_keystrokes(im, "muoons", &["m", "mu", "muo", "muô", "muôn", "muốn"]);
}

#[test]
fn test_telex_thuyeets_keystrokes() {
    let im = InputMethod::telex();
    verify_keystrokes(
        im,
        "thuyeets",
        &["t", "th", "thu", "thuy", "thuye", "thuyê", "thuyêt", "thuyết"],
    );
}

#[test]
fn test_vni_truong_keystrokes() {
    let im = InputMethod::vni();
    // t, r, u, 7(ư), o, 7(ơ), n, g, 2(huyền)
    verify_keystrokes(
        im,
        "tru7o7ng2",
        &["t", "tr", "tru", "trư", "trưo", "trươ", "trươn", "trương", "trường"],
    );
}

#[test]
fn test_telex_non_adjacent_repeated_tone_keys() {
    // Regression: non-adjacent repeated tone keys must override the earlier
    // ones instead of falling back to undo + literal key (bamboo "looixfsx").
    verify_keystrokes(
        InputMethod::telex(),
        "looixfsx",
        &["l", "lo", "lô", "lôi", "lỗi", "lồi", "lối", "lỗi"],
    );
    verify_keystrokes(InputMethod::telex(), "loixfx", &["l", "lo", "loi", "lõi", "lòi", "lõi"]);
    verify_keystrokes(InputMethod::telex(), "asfsx", &["a", "á", "à", "á", "ã"]);
}

#[test]
fn test_telex_adjacent_tone_key_undo() {
    // Adjacent duplicate tone keys are intentional undo patterns: the tone is
    // removed and the key is typed as a literal letter.
    verify_keystrokes(InputMethod::telex(), "ass", &["a", "á", "as"]);
    verify_keystrokes(InputMethod::telex(), "loiss", &["l", "lo", "loi", "lói", "lois"]);
    verify_keystrokes(InputMethod::telex(), "loiff", &["l", "lo", "loi", "lòi", "loif"]);
    verify_keystrokes(InputMethod::telex(), "loixx", &["l", "lo", "loi", "lõi", "loix"]);
}

#[test]
fn test_backspace_logic() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str("tieen", Mode::Vietnamese);
    assert_eq!(engine.output(), "tiên");

    engine.undo_last_key();
    assert_eq!(engine.output(), "tiê"); // Xóa 'n'

    engine.process_key('s', Mode::Vietnamese);
    assert_eq!(engine.output(), "tiế");
}
