//! A config change mid-word must leave the engine where a new engine with the new config would
//! be after typing the same keys: the output, raw keys, undo, backspace and restore all agree.
//!
//! Types every key sequence over a small alphabet, changes `extended_onsets` before each key,
//! and compares with a fresh engine. Slow in debug builds, so run it with:
//! `cargo test --release --test config_change_brute -- --ignored`
//! (`MAX_KEYS=5` checks longer sequences.)

use bamboo_core::{Config, Engine, InputMethod, LastWordForm, Mode, OutputOptions};

fn config(extended_onsets: bool, auto_correct: bool, free_tone_marking: bool) -> Config {
    Config::builder()
        .extended_onsets(extended_onsets)
        .auto_correct(auto_correct)
        .free_tone_marking(free_tone_marking)
        .build()
}

/// What the caller does right after the change.
fn after_change(engine: &mut Engine, action: usize) {
    match action {
        0 => {}
        1 => engine.undo_last_key(),
        2 => engine.remove_last_output_char(),
        3 => engine.restore_last_word_as(LastWordForm::Vietnamese),
        _ => engine.restore_last_word_as(LastWordForm::RawKeystrokes),
    }
}

fn state(engine: &Engine) -> (String, String) {
    (engine.output().into_owned(), engine.get_processed_str(OutputOptions::RAW))
}

#[test]
#[ignore = "slow; run with --release -- --ignored"]
fn a_change_mid_word_matches_a_fresh_engine() {
    let max_keys: usize = std::env::var("MAX_KEYS").map_or(4, |v| v.parse().unwrap());
    let methods = [(InputMethod::telex(), "fjwzaoesyrc"), (InputMethod::vni(), "fjwzao1265c")];
    let mut differ = Vec::new();
    for (method, alphabet) in methods {
        let im = || method.clone();
        let alphabet: Vec<char> = alphabet.chars().collect();
        for len in 2..=max_keys {
            for mut n in 0..alphabet.len().pow(len as u32) {
                let mut keys = String::with_capacity(len);
                for _ in 0..len {
                    keys.push(alphabet[n % alphabet.len()]);
                    n /= alphabet.len();
                }
                for (start, ac, free, split, action) in cases(len) {
                    let mut fresh = Engine::with_config(im(), config(!start, ac, free));
                    fresh.process_str(&keys[..split], Mode::Vietnamese);
                    after_change(&mut fresh, action);
                    fresh.process_str(&keys[split..], Mode::Vietnamese);

                    let mut engine = Engine::with_config(im(), config(start, ac, free));
                    engine.process_str(&keys[..split], Mode::Vietnamese);
                    engine.set_config(config(!start, ac, free));
                    after_change(&mut engine, action);
                    engine.process_str(&keys[split..], Mode::Vietnamese);

                    if state(&engine) != state(&fresh) {
                        differ.push(format!(
                            "{}|{} start={start} auto_correct={ac} free={free} action={action}: {:?}, fresh {:?}",
                            &keys[..split],
                            &keys[split..],
                            state(&engine),
                            state(&fresh)
                        ));
                    }
                }
            }
        }
    }
    assert!(
        differ.is_empty(),
        "{} differ, e.g. {:?}",
        differ.len(),
        &differ[..differ.len().min(5)]
    );
}

/// Every start setting, auto-correct, free tone marking, split point and action.
fn cases(len: usize) -> impl Iterator<Item = (bool, bool, bool, usize, usize)> {
    [true, false].into_iter().flat_map(move |start| {
        [true, false].into_iter().flat_map(move |ac| {
            [true, false].into_iter().flat_map(move |free| {
                (1..len).flat_map(move |split| (0..5).map(move |a| (start, ac, free, split, a)))
            })
        })
    })
}
