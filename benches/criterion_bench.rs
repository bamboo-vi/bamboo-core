//! Criterion micro/macro benchmarks for bamboo-core.
//!
//! Run with `cargo bench --bench criterion_bench`.
//! Reports are written to `target/criterion/`.

use bamboo_core::{Charset, Engine, InputMethodPreset, Mode, decode_charset, encode_charset};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group};
use std::hint::black_box;

fn fresh_engine() -> Engine {
    Engine::from_preset(InputMethodPreset::Telex)
}

fn warm_engine(keys: &str) -> Engine {
    let mut e = fresh_engine();
    e.process_str(keys, Mode::Vietnamese);
    e.reset();
    e
}

fn bench_dfa_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfa_hit");
    for word in ["tieengs", "vietj", "nguwowif", "dduwowngf", "khuyeens"] {
        group.bench_with_input(BenchmarkId::new("keystroke_stream", word), word, |b, w| {
            let mut e = warm_engine(w);
            b.iter(|| {
                e.reset();
                for ch in w.chars() {
                    e.process_key(black_box(ch), Mode::Vietnamese);
                }
                black_box(e.output_str());
            });
        });
    }
    group.finish();
}

fn bench_dfa_miss_slow(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfa_miss_slow");
    // Engine construction happens in setup, outside the timed region. Each
    // timed sample starts with an empty DFA and measures rule + JIT work only.
    group.bench_function("cold_tone_word", |b| {
        b.iter_batched_ref(
            fresh_engine,
            |e| {
                for ch in "nghieengs".chars() {
                    e.process_key(black_box(ch), Mode::Vietnamese);
                }
                black_box(e.output_str());
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("output");
    let mut e = warm_engine("tieengs");
    e.process_str("tieengs", Mode::Vietnamese);
    group.bench_function("output_str", |b| {
        b.iter(|| black_box(e.output_str()));
    });
    group.bench_function("output_cow", |b| {
        b.iter(|| black_box(e.output()));
    });
    group.finish();
}

fn bench_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta");
    group.bench_function("process_key_edit_per_key", |b| {
        let mut e = warm_engine("vietj");
        b.iter(|| {
            e.reset();
            for ch in "vietj".chars() {
                let d = e.process_key_edit(black_box(ch), Mode::Vietnamese);
                black_box(d);
            }
        });
    });
    group.finish();
}

fn bench_backspace(c: &mut Criterion) {
    let mut group = c.benchmark_group("backspace");
    group.bench_function("undo_last_key_x1", |b| {
        let mut e = warm_engine("tieengs");
        b.iter(|| {
            e.reset();
            e.process_str("tieengs", Mode::Vietnamese);
            e.undo_last_key();
            black_box(e.output_str());
        });
    });
    group.bench_function("undo_last_key_x3", |b| {
        let mut e = warm_engine("tieengs");
        b.iter(|| {
            e.reset();
            e.process_str("tieengs", Mode::Vietnamese);
            for _ in 0..3 {
                e.undo_last_key();
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_english(c: &mut Criterion) {
    let mut group = c.benchmark_group("english");
    group.bench_function("passthrough", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            e.reset();
            for ch in "getUserNameByIdentifier".chars() {
                e.process_key(black_box(ch), Mode::English);
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_uppercase(c: &mut Criterion) {
    let mut group = c.benchmark_group("uppercase");
    for word in ["tieengs", "TIEENGS", "Tieengs"] {
        group.bench_with_input(BenchmarkId::new("keystroke_stream", word), word, |b, w| {
            let mut e = warm_engine(&w.to_ascii_lowercase());
            b.iter(|| {
                e.reset();
                for ch in w.chars() {
                    e.process_key(black_box(ch), Mode::Vietnamese);
                }
                black_box(e.output_str());
            });
        });
    }
    group.finish();
}

fn bench_sentence(c: &mut Criterion) {
    let mut group = c.benchmark_group("sentence");
    group.bench_function("27_chars", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            e.reset();
            for ch in "tooi ddax tieengs Vieejt Nam".chars() {
                e.process_key(black_box(ch), Mode::Vietnamese);
            }
            black_box(e.output_str());
        });
    });
    group.bench_function("62_chars", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            e.reset();
            for ch in "Chufng ta ddaang soongs treen quoocs gia Vieejt Nam xinh depf vaaf tuwj doj"
                .chars()
            {
                e.process_key(black_box(ch), Mode::Vietnamese);
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_commit(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit");
    group.bench_function("commit_word", |b| {
        let mut e = warm_engine("tieengs");
        b.iter(|| {
            e.reset();
            e.process_str("tieengs", Mode::Vietnamese);
            e.commit();
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_charset(c: &mut Criterion) {
    const UNICODE: &str = "Đức Hưởng ăn quýt, Ỷ Lại uống trà ở Phố Hiến; trời thì ẩm ướt, ngoài sân cỏ úa dưới mưa phùn.";
    let viqr = encode_charset(Charset::Viqr, UNICODE).into_owned();
    let vni = encode_charset(Charset::VniWindows, UNICODE).into_owned();
    let mut group = c.benchmark_group("charset");

    group.bench_function("encode_viqr", |b| {
        b.iter(|| encode_charset(Charset::Viqr, black_box(UNICODE)));
    });
    group.bench_function("decode_viqr", |b| {
        b.iter(|| decode_charset(Charset::Viqr, black_box(&viqr)));
    });
    group.bench_function("decode_vni", |b| {
        b.iter(|| decode_charset(Charset::VniWindows, black_box(&vni)));
    });
    group.bench_function("decode_ascii_passthrough", |b| {
        b.iter(|| decode_charset(Charset::VniWindows, black_box("hello, world!")));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_dfa_hit,
    bench_dfa_miss_slow,
    bench_output,
    bench_delta,
    bench_backspace,
    bench_english,
    bench_uppercase,
    bench_sentence,
    bench_commit,
    bench_charset
);

fn main() {
    // `cargo test --all-targets` executes bench mains with no args: run a
    // fast smoke instead of the full criterion suite (`cargo bench` only,
    // which passes `--bench`).
    if !std::env::args().any(|a| a == "--bench") {
        println!("(smoke under `cargo test`; run `cargo bench` for the full criterion suite)");
        return;
    }
    benches();
}
