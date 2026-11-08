//! Four-engine scale benchmark: bamboo-core vs skey-engine, uvie, and vi.
//!
//! Run with `cargo bench --bench four_engine_scale`.
//!
//! skey and vi are batch transformers (no stateful per-key API for a whole
//! paragraph), so multi-word text is transformed word-by-word for them. The
//! incremental group re-transforms the accumulated prefix per key for skey,
//! matching its own internal benchmark, and uses vi's incremental buffer.
//!
//! uvie's committed buffer is a fixed `StackStr<128>`, so paragraphs are
//! flushed per word (what a host does when rendering committed words).
//!
//! `english_passthrough` and `random_keys` are intentionally not compared
//! across engines: their output legitimately differs per engine.

#![allow(deprecated)]

use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};
use criterion::{Criterion, Throughput, black_box};
use skey_engine::{Method, SkeyEngine};
use std::time::{Duration, Instant};
use uvie::UltraFastViEngine;
use vi::methods::{transform_buffer, transform_buffer_incremental};

const WORDS: &[(&str, &str)] = &[
    ("tieengs", "tiếng"),
    ("vieetj", "việt"),
    ("nguwowif", "người"),
    ("dduwowngf", "đường"),
    ("khuyeens", "khuyến"),
    ("khoong", "không"),
    ("dduowcj", "được"),
];

const ENGLISH: &str = "the quick brown fox jumps over the lazy dog and keeps running through the forest while the sun sets slowly behind the mountains";

fn phrase(words: usize) -> String {
    let mut text = String::with_capacity(words * 10);
    for i in 0..words {
        if i > 0 {
            text.push(' ');
        }
        text.push_str(WORDS[i % WORDS.len()].0);
    }
    text
}

fn expected_phrase(words: usize) -> String {
    let mut text = String::with_capacity(words * 8);
    for i in 0..words {
        if i > 0 {
            text.push(' ');
        }
        text.push_str(WORDS[i % WORDS.len()].1);
    }
    text
}

fn english_phrase(words: usize) -> String {
    let base: Vec<&str> = ENGLISH.split(' ').collect();
    let mut text = String::with_capacity(words * 6);
    for i in 0..words {
        if i > 0 {
            text.push(' ');
        }
        text.push_str(base[i % base.len()]);
    }
    text
}

/// Deterministic xorshift so the random-key scenario is reproducible.
fn random_text(chars: usize) -> String {
    let mut state = 0x1234_5678_9abc_def0u64;
    let mut text = String::with_capacity(chars);
    for i in 0..chars {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        if i > 0 && i % 8 == 0 {
            text.push(' ');
        } else {
            text.push(char::from(b'a' + (state % 26) as u8));
        }
    }
    text
}

fn skey_text(skey: &SkeyEngine, parts: &[&str]) -> String {
    let mut out = String::new();
    for (i, word) in parts.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&skey.transform(word));
    }
    out
}

fn vi_text(parts: &[&str]) -> String {
    let mut out = String::new();
    for (i, word) in parts.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        transform_buffer(&vi::TELEX, word.chars(), &mut out);
    }
    out
}

/// uvie's committed buffer is a fixed `StackStr<128>`, so flush it after each
/// word (what a host does when rendering committed words) instead of letting a
/// long paragraph overflow it.
fn uvie_word(engine: &mut UltraFastViEngine, word: &str, out: &mut String) {
    for ch in word.chars() {
        engine.feed(ch);
    }
    engine.feed(' ');
    out.push_str(engine.committed_text());
    out.push_str(engine.current_composing());
    engine.clear();
}

fn uvie_text(engine: &mut UltraFastViEngine, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split(' ') {
        uvie_word(engine, word, &mut out);
    }
    out.pop();
    out
}

fn bench_batch(c: &mut Criterion, name: &str, text: &str) {
    let parts: Vec<&str> = text.split(' ').collect();
    let skey = SkeyEngine::new(Method::Telex);

    let mut group = c.benchmark_group(name);
    group.throughput(Throughput::Elements(text.chars().count() as u64));

    group.bench_function("bamboo", |b| {
        let mut engine = Engine::new(InputMethod::telex());
        engine.warm_up();
        b.iter(|| {
            engine.reset();
            engine.process_str(black_box(text), Mode::Vietnamese);
            black_box(engine.get_processed_str(OutputOptions::FULL_TEXT));
        });
    });
    group.bench_function("uvie", |b| {
        let mut engine = UltraFastViEngine::new();
        let mut out = String::with_capacity(text.len());
        b.iter(|| {
            out.clear();
            for word in &parts {
                uvie_word(&mut engine, black_box(word), &mut out);
            }
            black_box(&out);
        });
    });
    group.bench_function("skey", |b| {
        let mut out = String::with_capacity(text.len());
        b.iter(|| {
            out.clear();
            for (i, word) in parts.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                out.push_str(&skey.transform(black_box(*word)));
            }
            black_box(&out);
        });
    });
    group.bench_function("vi", |b| {
        let mut out = String::with_capacity(text.len());
        b.iter(|| {
            out.clear();
            for (i, word) in parts.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                transform_buffer(&vi::TELEX, black_box(*word).chars(), &mut out);
            }
            black_box(&out);
        });
    });
    group.finish();
}

fn bench_incremental(c: &mut Criterion, name: &str, text: &str) {
    let mut group = c.benchmark_group(name);
    group.throughput(Throughput::Elements(text.chars().count() as u64));

    group.bench_function("bamboo", |b| {
        let mut engine = Engine::new(InputMethod::telex());
        engine.warm_up();
        b.iter(|| {
            engine.reset();
            for ch in text.chars() {
                engine.process_key(black_box(ch), Mode::Vietnamese);
                black_box(engine.output_str());
            }
        });
    });
    group.bench_function("uvie", |b| {
        let mut engine = UltraFastViEngine::new();
        b.iter(|| {
            engine.clear();
            for ch in text.chars() {
                black_box(engine.feed(ch));
            }
        });
    });
    group.bench_function("skey", |b| {
        let skey = SkeyEngine::new(Method::Telex);
        let mut word = String::with_capacity(16);
        b.iter(|| {
            word.clear();
            for ch in text.chars() {
                if ch == ' ' {
                    word.clear();
                } else {
                    word.push(ch);
                    black_box(skey.transform(&word));
                }
            }
        });
    });
    group.bench_function("vi", |b| {
        let mut buffer = transform_buffer_incremental(&vi::TELEX);
        b.iter(|| {
            buffer.clear();
            for ch in text.chars() {
                if ch == ' ' {
                    buffer.clear();
                } else {
                    black_box(buffer.push(ch));
                }
            }
        });
    });
    group.finish();
}

fn bench_deep_syllables(c: &mut Criterion) {
    for (input, expected) in [("nghieengs", "nghiếng"), ("dduwowcj", "được")] {
        let skey = SkeyEngine::new(Method::Telex);
        assert_eq!(skey.transform(input), expected);
        assert_eq!(vi_text(&[input]), expected);
        let mut uvie = UltraFastViEngine::new();
        assert_eq!(uvie_text(&mut uvie, input), expected);
        let mut bamboo = Engine::new(InputMethod::telex());
        bamboo.process_str(input, Mode::Vietnamese);
        assert_eq!(bamboo.get_processed_str(OutputOptions::FULL_TEXT), expected);

        let mut group = c.benchmark_group(format!("deep_syllable/{input}"));
        group.throughput(Throughput::Elements(input.chars().count() as u64));

        group.bench_function("bamboo", |b| {
            let mut engine = Engine::new(InputMethod::telex());
            engine.warm_up();
            b.iter(|| {
                engine.reset();
                for ch in input.chars() {
                    engine.process_key(black_box(ch), Mode::Vietnamese);
                    black_box(engine.output_str());
                }
            });
        });
        group.bench_function("uvie", |b| {
            let mut engine = UltraFastViEngine::new();
            b.iter(|| {
                engine.clear();
                for ch in input.chars() {
                    black_box(engine.feed(ch));
                }
            });
        });
        group.bench_function("skey", |b| {
            let mut acc = String::with_capacity(input.len());
            b.iter(|| {
                acc.clear();
                for ch in input.chars() {
                    acc.push(ch);
                    black_box(skey.transform(&acc));
                }
            });
        });
        group.bench_function("vi", |b| {
            let mut buffer = transform_buffer_incremental(&vi::TELEX);
            b.iter(|| {
                buffer.clear();
                for ch in input.chars() {
                    black_box(buffer.push(ch));
                }
            });
        });
        group.finish();
    }
}

fn bench_backspace_burst(c: &mut Criterion) {
    let word = "dduwowcj";
    let keys = word.chars().count();
    let mut group = c.benchmark_group(format!("backspace_burst/{word}"));
    group.throughput(Throughput::Elements((keys * 2) as u64));

    group.bench_function("bamboo", |b| {
        let mut engine = Engine::new(InputMethod::telex());
        engine.warm_up();
        b.iter(|| {
            engine.reset();
            engine.process_str(word, Mode::Vietnamese);
            for _ in 0..keys {
                engine.undo_last_key();
            }
            black_box(engine.output_str());
        });
    });
    group.bench_function("uvie", |b| {
        let mut engine = UltraFastViEngine::new();
        b.iter(|| {
            engine.clear();
            for ch in word.chars() {
                engine.feed(ch);
            }
            for _ in 0..keys {
                black_box(engine.backspace());
            }
        });
    });
    group.bench_function("skey", |b| {
        let skey = SkeyEngine::new(Method::Telex);
        b.iter(|| {
            for end in (0..=word.len()).rev() {
                black_box(skey.transform(&word[..end]));
            }
        });
    });
    group.bench_function("vi", |b| {
        let mut out = String::with_capacity(word.len());
        b.iter(|| {
            for end in (0..=word.len()).rev() {
                out.clear();
                transform_buffer(&vi::TELEX, word[..end].chars(), &mut out);
                black_box(&out);
            }
        });
    });
    group.finish();
}

fn bench_cold_start(c: &mut Criterion) {
    let word = "dduwowcj";
    let mut group = c.benchmark_group("cold_start");
    group.throughput(Throughput::Elements(word.chars().count() as u64));

    group.bench_function("bamboo", |b| {
        b.iter(|| {
            let mut engine = Engine::new(InputMethod::telex());
            engine.process_str(black_box(word), Mode::Vietnamese);
            black_box(engine.get_processed_str(OutputOptions::FULL_TEXT));
        });
    });
    group.bench_function("uvie", |b| {
        b.iter(|| {
            let mut engine = UltraFastViEngine::new();
            for ch in word.chars() {
                engine.feed(ch);
            }
            black_box(engine.current_output());
        });
    });
    group.bench_function("skey", |b| {
        b.iter(|| {
            let skey = SkeyEngine::new(Method::Telex);
            black_box(skey.transform(black_box(word)));
        });
    });
    group.bench_function("vi", |b| {
        let mut out = String::with_capacity(word.len());
        b.iter(|| {
            out.clear();
            transform_buffer(&vi::TELEX, black_box(word).chars(), &mut out);
            black_box(&out);
        });
    });
    group.finish();
}

fn bench_long_session() {
    const TARGET_CHARS: usize = 1_000_000;
    let base = phrase(448);
    let repeats = TARGET_CHARS.div_ceil(base.chars().count());
    let mut text = format!("{base} ").repeat(repeats);
    text.pop();
    let mut expected = format!("{} ", expected_phrase(448)).repeat(repeats);
    expected.pop();
    let parts: Vec<&str> = text.split(' ').collect();
    let chars = text.chars().count();

    println!("\n=== Long session ({chars} chars) ===");
    println!("{:<8} {:>12} {:>10} {:>12}", "engine", "total ms", "ns/char", "output");

    let report = |engine: &str, elapsed: Duration, output: &str| {
        println!(
            "{:<8} {:>12.2} {:>10.1} {:>12}",
            engine,
            elapsed.as_secs_f64() * 1000.0,
            elapsed.as_nanos() as f64 / chars as f64,
            output.len()
        );
    };

    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let start = Instant::now();
    bamboo.process_str(&text, Mode::Vietnamese);
    let out = bamboo.get_processed_str(OutputOptions::FULL_TEXT);
    report("bamboo", start.elapsed(), &out);
    assert_eq!(out, expected);

    let mut uvie = UltraFastViEngine::new();
    let start = Instant::now();
    let out = uvie_text(&mut uvie, &text);
    report("uvie", start.elapsed(), &out);
    assert_eq!(out, expected);

    let skey = SkeyEngine::new(Method::Telex);
    let start = Instant::now();
    let out = skey_text(&skey, &parts);
    report("skey", start.elapsed(), &out);
    assert_eq!(out, expected);

    let start = Instant::now();
    let out = vi_text(&parts);
    report("vi", start.elapsed(), &out);
    assert_eq!(out, expected);
}

fn main() {
    if !std::env::args().any(|arg| arg == "--bench") {
        println!("Run `cargo bench --bench four_engine_scale` for the scale comparison.");
        return;
    }

    let mut criterion = Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));

    // Short/deep cases first so they are measured before the heavy paragraph
    // groups heat the CPU; the long session runs last.
    bench_deep_syllables(&mut criterion);
    bench_backspace_burst(&mut criterion);
    bench_cold_start(&mut criterion);
    bench_batch(&mut criterion, "english_passthrough", &english_phrase(64));
    bench_batch(&mut criterion, "random_keys", &random_text(256));
    for words in [7usize, 28, 112] {
        bench_incremental(
            &mut criterion,
            &format!("paragraph_incremental/{words}_words"),
            &phrase(words),
        );
    }
    for words in [7usize, 28, 112, 448] {
        let text = phrase(words);
        let expected = expected_phrase(words);
        let parts: Vec<&str> = text.split(' ').collect();
        let skey = SkeyEngine::new(Method::Telex);
        let mut uvie = UltraFastViEngine::new();
        let mut bamboo = Engine::new(InputMethod::telex());
        bamboo.process_str(&text, Mode::Vietnamese);
        assert_eq!(skey_text(&skey, &parts), expected, "skey output");
        assert_eq!(vi_text(&parts), expected, "vi output");
        assert_eq!(uvie_text(&mut uvie, &text), expected, "uvie output");
        assert_eq!(bamboo.get_processed_str(OutputOptions::FULL_TEXT), expected, "bamboo output");
        bench_batch(&mut criterion, &format!("paragraph_batch/{words}_words"), &text);
    }
    criterion.final_summary();

    bench_long_session();
}
