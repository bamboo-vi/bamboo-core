//! Telex must not turn ordinary English words into Vietnamese.
//!
//! Regression guard for issue #31: a refused mark key after a valid syllable
//! keeps the composition, so letters that double as Telex mark keys must not
//! drag English words into Vietnamese.
//!
//! Corpus: the 10,000 most common English words from Google's Trillion Word
//! Corpus, vendored from <https://github.com/first20hours/google-10000-english>
//! so the check runs offline. Run the report with:
//! `cargo test --test english_telex_corpus -- --ignored --nocapture`

use bamboo_core::{Config, Engine, InputMethod, Mode, OutputOptions};

#[test]
#[ignore = "diagnostic corpus report, run explicitly"]
fn english_words_survive_telex_autocorrect() {
    let words: Vec<&str> = include_str!("data/google-10000-english.txt")
        .lines()
        .filter(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase()))
        .collect();

    let mut wrong = 0usize;
    let mut samples = Vec::new();
    for w in &words {
        let mut e = Engine::with_config(InputMethod::telex(), Config::default());
        let input = format!("{w} ");
        e.process_str(&input, Mode::Vietnamese);
        let out = e.get_processed_str(OutputOptions::FULL_TEXT);
        if out != input {
            wrong += 1;
            if samples.len() < 10 {
                samples.push(format!("{w} -> {out:?}"));
            }
        }
    }

    let pct = 100.0 * (words.len() - wrong) as f64 / words.len() as f64;
    println!("{} words, {wrong} changed, {pct:.2}% preserved", words.len());
    for s in &samples {
        println!("  {s}");
    }

    // 92.50% measured. The remaining words are structurally ambiguous: a
    // refused tone key after a valid tone-carrying syllable (`việt` + f and
    // `mít` + r are indistinguishable at keystroke time).
    assert!(pct >= 92.0, "English preservation regressed to {pct:.2}%");
}
