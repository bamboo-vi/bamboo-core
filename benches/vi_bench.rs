use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};
use criterion::{BenchmarkId, Criterion, Throughput};
use skey_engine::{Method, SkeyEngine};
use std::hint::black_box;
use std::time::Duration;
use uvie::UltraFastViEngine;
use vi::methods::transform_buffer;

const CASES: &[(&str, &str, &str)] = &[
    ("tieengs", "tieengs", "tiếng"),
    ("vieetj", "vieetj", "việt"),
    ("nguwowif", "nguwowif", "người"),
    ("dduwowngf", "dduwowngf", "đường"),
    ("khuyeens", "khuyeens", "khuyến"),
    ("khoong", "khoong", "không"),
    ("dduowcj", "dduowcj", "được"),
];

fn main() {
    if !std::env::args().any(|arg| arg == "--bench") {
        println!("Run `cargo bench --bench vi_bench` for the four-engine comparison.");
        return;
    }

    let mut criterion = Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));
    bench_engines(&mut criterion);
    criterion.final_summary();
}

fn bench_engines(criterion: &mut Criterion) {
    let mut bamboo = Engine::new(InputMethod::telex());
    let mut uvie = UltraFastViEngine::new();
    let skey = SkeyEngine::new(Method::Telex);
    let mut vi_output = String::with_capacity(128);

    // Warm each implementation before comparing steady-state text transforms.
    for &(_, input, _) in CASES {
        bamboo.reset();
        bamboo.process_str(input, Mode::Vietnamese);
        uvie.clear();
        for ch in input.chars() {
            uvie.feed(ch);
        }
        let _ = skey.transform(input);
        vi_output.clear();
        transform_buffer(&vi::TELEX, input.chars(), &mut vi_output);
    }

    for &(case, input, expected) in CASES {
        bamboo.reset();
        bamboo.process_str(input, Mode::Vietnamese);
        let bamboo_output = bamboo.get_processed_str(OutputOptions::FULL_TEXT);

        uvie.clear();
        for ch in input.chars() {
            uvie.feed(ch);
        }
        let uvie_output = uvie.current_output().to_string();

        vi_output.clear();
        transform_buffer(&vi::TELEX, input.chars(), &mut vi_output);
        let skey_output = skey.transform(input);
        assert_eq!(bamboo_output, expected, "Bamboo Telex result for {case}");
        assert_eq!(uvie_output, expected, "Uvie Telex result for {case}");
        assert_eq!(vi_output, expected, "vi Telex result for {case}");
        assert_eq!(skey_output, expected, "skey Telex result for {case}");

        let mut group = criterion.benchmark_group(format!("text_transform/{case}"));
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_function(BenchmarkId::new("bamboo", case), |b| {
            b.iter(|| {
                bamboo.reset();
                bamboo.process_str(black_box(input), Mode::Vietnamese);
                black_box(bamboo.get_processed_str(OutputOptions::FULL_TEXT));
            });
        });
        group.bench_function(BenchmarkId::new("uvie", case), |b| {
            b.iter(|| {
                uvie.clear();
                for ch in black_box(input).chars() {
                    uvie.feed(ch);
                }
                black_box(uvie.current_output().to_string());
            });
        });
        group.bench_function(BenchmarkId::new("vi", case), |b| {
            b.iter(|| {
                vi_output.clear();
                transform_buffer(&vi::TELEX, black_box(input).chars(), &mut vi_output);
                black_box(vi_output.clone());
            });
        });
        group.bench_function(BenchmarkId::new("skey", case), |b| {
            b.iter(|| black_box(skey.transform(black_box(input))));
        });
        group.finish();
    }
}
