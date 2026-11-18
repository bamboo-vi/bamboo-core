use bamboo_core::{Config, Engine, InputMethod, LastWordForm, Mode, OutputOptions, W2uMode};

fn config(extended_onsets: bool, auto_correct: bool) -> Config {
    Config::builder().extended_onsets(extended_onsets).auto_correct(auto_correct).build()
}

fn new_engine(im: InputMethod, extended_onsets: bool) -> Engine {
    Engine::with_config(im, config(extended_onsets, true))
}

/// Types `keys` in Telex: `<` deletes the last character, `+` and `-` turn extended onsets on
/// and off, and `!` restores the raw keys of the word.
fn typed_with(start: bool, auto_correct: bool, keys: &str) -> String {
    let mut engine = Engine::with_config(InputMethod::telex(), config(start, auto_correct));
    for key in keys.chars() {
        match key {
            '<' => engine.remove_last_output_char(),
            '+' => engine.set_config(config(true, auto_correct)),
            '-' => engine.set_config(config(false, auto_correct)),
            '!' => engine.restore_last_word_as(LastWordForm::RawKeystrokes),
            _ => engine.process_key(key, Mode::Vietnamese),
        }
    }
    engine.output().into_owned()
}

fn typed(start: bool, keys: &str) -> String {
    typed_with(start, true, keys)
}

fn typed_in(im: InputMethod, extended_onsets: bool, keys: &str) -> String {
    let mut engine = new_engine(im, extended_onsets);
    engine.process_str(keys, Mode::Vietnamese);
    engine.output().into_owned()
}

#[test]
fn on_by_default() {
    assert!(Config::default().extended_onsets);
    assert!(Engine::new(InputMethod::telex()).config().extended_onsets);
    assert_eq!(typed_in(InputMethod::telex(), true, "jaayj"), "jậy");
}

#[test]
fn on_takes_f_j_and_w() {
    for (keys, want) in [("faayj", "fậy"), ("jaayj", "jậy"), ("waayj", "wậy"), ("fair", "fải")]
    {
        assert_eq!(typed(true, keys), want);
    }
    assert_eq!(typed(true, "just"), "jút");
    assert_eq!(typed(true, "ferry"), "fery");
}

#[test]
fn off_keeps_f_j_and_w_as_typed() {
    for keys in ["faayj", "jaayj", "waayj", "fair", "just", "jussive", "ferry"] {
        assert_eq!(typed(false, keys), keys);
    }
}

#[test]
fn off_still_takes_z_and_vietnamese() {
    for on in [true, false] {
        assert_eq!(typed(on, "zaayj"), "zậy");
        assert_eq!(typed(on, "vieetj"), "việt");
        assert_eq!(typed(on, "tieengs"), "tiếng");
    }
}

#[test]
fn off_reports_the_word_invalid() {
    // Front ends that keep auto-correct off still ask whether the word is Vietnamese.
    for auto_correct in [true, false] {
        for (on, want) in [(true, true), (false, false)] {
            let mut engine = Engine::with_config(InputMethod::telex(), config(on, auto_correct));
            engine.process_str("jaa", Mode::Vietnamese);
            assert_eq!(engine.is_valid_word(), want, "on={on} auto_correct={auto_correct}");
        }
    }
}

#[test]
fn uppercase_follows_the_setting() {
    assert_eq!(typed(true, "JAAYJ"), "JẬY");
    assert_eq!(typed(true, "Jaayj"), "Jậy");
    assert_eq!(typed(false, "JAAYJ"), "JAAYJ");
    assert_eq!(typed(false, "Jaayj"), "Jaayj");
}

#[test]
fn without_auto_correct_the_setting_changes_nothing() {
    // Without auto-correct an invalid syllable still takes marks, so f, j and w do too.
    for keys in ["jaayj", "faayj", "waayj", "just", "jussive", "ferry", "vieetj"] {
        assert_eq!(typed_with(true, false, keys), typed_with(false, false, keys), "{keys}");
    }
    assert_eq!(typed_with(false, false, "jaayj"), "jậy");
}

#[test]
fn vni_follows_the_setting() {
    assert_eq!(typed_in(InputMethod::vni(), true, "ja65"), "jậ");
    assert_eq!(typed_in(InputMethod::vni(), true, "JA65"), "JẬ");
    assert_eq!(typed_in(InputMethod::vni(), false, "ja65"), "ja65");
    assert_eq!(typed_in(InputMethod::vni(), false, "fa61"), "fa61");
    for on in [true, false] {
        assert_eq!(typed_in(InputMethod::vni(), on, "za65"), "zậ");
        assert_eq!(typed_in(InputMethod::vni(), on, "vie65t"), "việt");
    }
}

#[test]
fn viqr_and_custom_follow_the_setting() {
    assert_eq!(typed_in(InputMethod::viqr(), true, "ja^."), "jậ");
    assert_eq!(typed_in(InputMethod::viqr(), false, "ja^."), "ja^.");
    assert_eq!(typed_in(InputMethod::viqr(), false, "za^."), "zậ");

    let custom = || InputMethod::from_definition("Custom", [("q", "DauSac"), ("a", "A_Â")]);
    assert_eq!(typed_in(custom(), true, "jaaq"), "jấ");
    assert_eq!(typed_in(custom(), false, "jaaq"), "jaaq");
    assert_eq!(typed_in(custom(), false, "baaq"), "bấ");
}

#[test]
fn backspace_follows_the_setting() {
    assert_eq!(typed(true, "jaayj<"), "jậ");
    assert_eq!(typed(true, "jaayj<y"), "jậy");
    assert_eq!(typed(false, "jaayj<"), "jaay");
    assert_eq!(typed(false, "zaayj<"), "zậ");
}

#[test]
fn raw_keys_survive_either_way() {
    for on in [true, false] {
        let mut engine = new_engine(InputMethod::telex(), on);
        engine.process_str("jaayj", Mode::Vietnamese);
        assert_eq!(engine.get_processed_str(OutputOptions::RAW), "jaayj");
        assert_eq!(typed(on, "jaayj!"), "jaayj");
    }
}

/// Types `before`, turns the setting `now_on`, types `rest`. Also returns what an engine with
/// the new setting from the start gives for all the keys.
fn switched(
    im: fn() -> InputMethod,
    auto_correct: bool,
    before: &str,
    now_on: bool,
    rest: &str,
) -> (String, String) {
    let mut engine = Engine::with_config(im(), config(!now_on, auto_correct));
    engine.process_str(before, Mode::Vietnamese);
    engine.set_config(config(now_on, auto_correct));
    engine.process_str(rest, Mode::Vietnamese);
    let mut fresh = Engine::with_config(im(), config(now_on, auto_correct));
    fresh.process_str(&format!("{before}{rest}"), Mode::Vietnamese);
    (engine.output().into_owned(), fresh.output().into_owned())
}

#[test]
fn a_change_mid_word_retypes_the_word() {
    for (im, auto_correct, before, now_on, rest, want) in [
        (InputMethod::telex as fn() -> InputMethod, true, "jaay", false, "j", "jaayj"),
        (InputMethod::telex, true, "jaay", true, "j", "jậy"),
        (InputMethod::telex, true, "fofa", false, "f", "fofaf"),
        (InputMethod::telex, false, "fac", false, "s", "fác"),
        (InputMethod::telex, false, "fac", true, "s", "fác"),
        (InputMethod::telex, true, "juss", false, "ive", "jussive"),
        (InputMethod::telex, true, "juss", true, "ive", "jusive"),
        (InputMethod::vni, true, "ja6", false, "5", "ja65"),
        (InputMethod::vni, true, "ja6", true, "5", "jậ"),
        (InputMethod::vni, false, "fo", false, "1a", "fóa"),
    ] {
        let (got, fresh) = switched(im, auto_correct, before, now_on, rest);
        assert_eq!(got, want, "{before}|{rest} now_on={now_on} auto_correct={auto_correct}");
        assert_eq!(got, fresh, "{before}|{rest}");
    }
}

#[test]
fn the_output_waits_for_the_next_key() {
    assert_eq!(typed(true, "jaayj-"), "jậy");
    assert_eq!(typed(false, "jaayj+"), "jaayj");
}

#[test]
fn the_next_edit_covers_what_the_change_did() {
    let mut engine = new_engine(InputMethod::telex(), true);
    let mut shown = String::new();
    let mut apply = |engine: &mut Engine, key: char| {
        let edit = engine.process_key_edit(key, Mode::Vietnamese);
        let keep = shown.chars().count() - edit.delete_chars;
        shown = shown.chars().take(keep).collect::<String>() + edit.insert;
    };
    for key in "jaay".chars() {
        apply(&mut engine, key);
    }
    engine.set_config(config(false, true));
    apply(&mut engine, 'j');
    assert_eq!(engine.output(), "jaayj");
    assert_eq!(shown, "jaayj");
}

#[test]
fn rebuilt_diacritic_survives_config_change() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.rebuild_from_text("café");
    assert_eq!(engine.output(), "café");
    engine.set_config(config(false, true));
    engine.process_key(' ', Mode::Vietnamese);
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "café ");
}

#[test]
fn english_aa_survives_config_change() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.process_str("aa", Mode::English);
    assert_eq!(engine.output(), "aa");
    engine.set_config(config(false, true));
    engine.process_key('b', Mode::English);
    assert_eq!(engine.output(), "aab");
}

#[test]
fn replay_boundary_preserves_host_text() {
    for tracking in [true, false] {
        let mut engine = Engine::with_config(
            InputMethod::telex(),
            Config::builder()
                .bracket_mode(bamboo_core::BracketMode::Everywhere)
                .auto_correct(false)
                .build(),
        );
        engine.set_full_text_tracking(tracking);
        let mut shown = String::new();
        for key in "a[".chars() {
            let edit = engine.process_key_edit(key, Mode::Vietnamese);
            let keep = shown.chars().count() - edit.delete_chars;
            shown = shown.chars().take(keep).collect::<String>() + edit.insert;
        }
        assert_eq!(shown, "aơ");
        engine.set_config(Config::builder().auto_correct(false).build());
        let edit = engine.process_key_edit('b', Mode::Vietnamese);
        let keep = shown.chars().count() - edit.delete_chars;
        shown = shown.chars().take(keep).collect::<String>() + edit.insert;
        assert_eq!(shown, "aơb", "tracking={tracking}");
        assert_eq!(engine.output(), "aơb", "tracking={tracking}");
        assert_eq!(engine.committed_text_len(), 0, "tracking={tracking}");
        assert_eq!(
            engine.get_processed_str(OutputOptions::FULL_TEXT),
            "aơb",
            "tracking={tracking}"
        );
        engine.set_config(config(true, false));
        engine.process_key('c', Mode::Vietnamese);
        assert_eq!(engine.output(), "aơbc", "tracking={tracking}");
        assert_eq!(engine.committed_text_len(), 0, "tracking={tracking}");
    }
}

#[test]
fn unsafe_replay_preserves_undo_and_restore() {
    for tracking in [false, true] {
        let make_engine = || {
            let mut e = Engine::with_config(
                InputMethod::telex(),
                Config::builder()
                    .bracket_mode(bamboo_core::BracketMode::Everywhere)
                    .auto_correct(false)
                    .build(),
            );
            e.set_full_text_tracking(tracking);
            e.process_str("a[", Mode::Vietnamese);
            e.set_config(Config::builder().auto_correct(false).build());
            e
        };
        let mut e = make_engine();
        e.undo_last_key();
        assert_eq!(e.output(), "a");
        e.undo_last_key();
        assert_eq!(e.output(), "");

        let mut e = make_engine();
        e.restore_last_word_as(LastWordForm::Vietnamese);
        assert_eq!(e.output(), "aơ");
        e.undo_last_key();
        assert_eq!(e.output(), "a");
    }
}

#[test]
fn skipped_replay_discards_stale_syllable_hint() {
    let mut e = Engine::with_config(InputMethod::telex(), config(true, false));
    e.rebuild_from_text("j");
    e.process_str("af", Mode::Vietnamese);
    e.set_config(config(false, false));
    e.process_str("wus", Mode::Vietnamese);
    assert_eq!(e.output(), "jằú");
}

#[test]
fn empty_restore_keeps_new_word_replayable() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.restore_last_word_as(LastWordForm::Vietnamese);
    engine.process_str("jaay", Mode::Vietnamese);
    engine.set_config(config(false, true));
    engine.process_key('j', Mode::Vietnamese);
    assert_eq!(engine.output(), "jaayj");
}

#[test]
fn uppercase_keys_retype_the_word() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.process_str("JAAY", Mode::Vietnamese);
    engine.set_config(config(false, true));
    engine.process_key('J', Mode::Vietnamese);
    assert_eq!(engine.output(), "JAAYJ");

    let mut engine = new_engine(InputMethod::telex(), true);
    engine.process_str("Jaay", Mode::Vietnamese);
    engine.set_config(config(false, true));
    engine.process_key('j', Mode::Vietnamese);
    assert_eq!(engine.output(), "Jaayj");
}

#[test]
fn a_second_change_retapes_with_the_latest_config() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.process_str("jaay", Mode::Vietnamese);
    engine.set_config(config(false, true));
    engine.set_config(config(true, true));
    engine.process_key('j', Mode::Vietnamese);
    assert_eq!(engine.output(), "jậy");
}

/// Simulates the delta-based host from the README: `app` is the committed application text
/// and `mirror` the preedit the host keeps from [`Engine::process_key_edit`].
fn host_key(app: &mut String, mirror: &mut String, engine: &mut Engine, key: char) {
    let shown = mirror.clone();
    let edit = engine.process_key_edit(key, Mode::Vietnamese);
    if edit.committed {
        // Forced boundary: the host flushes the mirror it shows and starts a fresh preedit.
        app.push_str(&shown);
        mirror.clear();
        mirror.push_str(edit.insert);
        return;
    }
    let keep = mirror.chars().count() - edit.delete_chars;
    *mirror = mirror.chars().take(keep).collect::<String>() + edit.insert;
    if key == ' ' {
        // Word break: the host commits its mirror plus the key itself.
        app.push_str(&shown);
        app.push(key);
        mirror.clear();
    }
}

#[test]
fn a_word_break_commits_the_preedit_the_host_shows() {
    // The key after the change ends the word, so the host commits the preedit it shows and
    // the composition is kept as it is instead of being re-typed (see `set_config`).
    let mut engine = new_engine(InputMethod::telex(), true);
    let (mut app, mut mirror) = (String::new(), String::new());
    for key in "jaay".chars() {
        host_key(&mut app, &mut mirror, &mut engine, key);
    }
    assert_eq!(mirror, "jây");
    engine.set_config(config(false, true));
    host_key(&mut app, &mut mirror, &mut engine, ' ');
    assert_eq!(format!("{app}{mirror}"), engine.get_processed_str(OutputOptions::FULL_TEXT));
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "jây ");
}

#[test]
fn a_forced_boundary_commits_the_preedit_the_host_shows() {
    let w2u = |mode| Config { w2u_mode: mode, auto_correct: false, ..Config::default() };
    for on in [true, false] {
        let mut engine = Engine::with_config(
            InputMethod::telex(),
            w2u(if on { W2uMode::Everywhere } else { W2uMode::Disabled }),
        );
        let (mut app, mut mirror) = (String::new(), String::new());
        for key in format!("{}w", "b".repeat(31)).chars() {
            host_key(&mut app, &mut mirror, &mut engine, key);
        }
        engine.set_config(w2u(if on { W2uMode::Disabled } else { W2uMode::Everywhere }));
        // The 33rd slot does not fit: the word ends as a signaled boundary.
        host_key(&mut app, &mut mirror, &mut engine, 'z');
        assert_eq!(format!("{app}{mirror}"), engine.get_processed_str(OutputOptions::FULL_TEXT));
    }
}

#[test]
fn undo_and_restore_after_a_change() {
    let mut engine = new_engine(InputMethod::telex(), true);
    engine.process_str("jaayj", Mode::Vietnamese);
    engine.set_config(config(false, true));
    engine.undo_last_key();
    assert_eq!(engine.output(), "jaay");

    let mut engine = new_engine(InputMethod::telex(), false);
    engine.process_str("jaa", Mode::Vietnamese);
    engine.set_config(config(true, true));
    engine.restore_last_word_as(LastWordForm::Vietnamese);
    assert_eq!(engine.output(), "jâ");
    engine.process_key('s', Mode::Vietnamese);
    assert_eq!(engine.output(), "jấ");
    assert_eq!(engine.get_processed_str(OutputOptions::RAW), "jaas");
}

#[test]
fn committed_text_stays_as_typed() {
    for tracking in [true, false] {
        let mut engine = new_engine(InputMethod::telex(), true);
        engine.set_full_text_tracking(tracking);
        engine.process_str("jaayj ja", Mode::Vietnamese);
        engine.set_config(config(false, true));
        engine.process_key('a', Mode::Vietnamese);
        let want = if tracking { "jậy jaa" } else { "jaa" };
        assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), want);
    }
}

#[test]
fn any_config_change_retypes_the_word() {
    let w2u = |mode| Config { w2u_mode: mode, ..Config::default() };
    let mut engine = Engine::with_config(InputMethod::telex(), w2u(W2uMode::Disabled));
    engine.process_str("bw", Mode::Vietnamese);
    engine.set_config(w2u(W2uMode::Everywhere));
    engine.process_key('a', Mode::Vietnamese);
    let mut fresh = Engine::with_config(InputMethod::telex(), w2u(W2uMode::Everywhere));
    fresh.process_str("bwa", Mode::Vietnamese);
    assert_eq!(engine.output(), fresh.output());
}

#[test]
fn engines_keep_their_own_setting() {
    let mut on = new_engine(InputMethod::telex(), true);
    let mut off = new_engine(InputMethod::telex(), false);
    for key in "jaayj".chars() {
        on.process_key(key, Mode::Vietnamese);
        off.process_key(key, Mode::Vietnamese);
    }
    assert_eq!(on.output(), "jậy");
    assert_eq!(off.output(), "jaayj");
}

#[cfg(feature = "parallel")]
#[test]
fn batch_uses_the_default() {
    let out = bamboo_core::parallel::process_batch(
        &["jaayj", "vieetj"],
        &InputMethod::telex(),
        Mode::Vietnamese,
    );
    assert_eq!(out, ["jậy", "việt"]);
}
