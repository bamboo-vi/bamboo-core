use bamboo_core::{Engine, InputMethodPreset, Mode, OutputOptions};
use std::{hint::black_box, time::Instant};
fn engine(im: &str) -> Engine {
    let p = match im {
        "vni" => InputMethodPreset::Vni,
        "viqr" => InputMethodPreset::Viqr,
        _ => InputMethodPreset::Telex,
    };
    Engine::from_preset(p)
}
fn main() {
    let cases = [
        ("telex", "tieengs", "telex", Mode::Vietnamese),
        ("long", "nguwowif", "telex", Mode::Vietnamese),
        ("english", "getUserNameByIdentifier", "telex", Mode::English),
        ("vni", "tie61ng", "vni", Mode::Vietnamese),
        ("viqr", "tie^'ng", "viqr", Mode::Vietnamese),
    ];
    let n = 30_000;
    for (name, word, im, mode) in cases {
        let mut e = engine(im);
        let keys: Vec<char> = word.chars().collect();
        for _ in 0..1000 {
            e.reset();
            for &k in &keys {
                e.process_key(k, mode);
                black_box(e.output_str());
            }
        }
        let mut sample = Vec::new();
        for _ in 0..5 {
            let t = Instant::now();
            for _ in 0..n {
                e.reset();
                for &k in &keys {
                    e.process_key(k, mode);
                    black_box(e.output_str());
                }
            }
            sample.push(t.elapsed().as_nanos() as f64 / n as f64);
        }
        sample.sort_by(f64::total_cmp);
        e.reset();
        for &k in &keys {
            e.process_key(k, mode);
        }
        println!(
            "BENCH\t{name}\t{:.1}\t{}",
            sample[2],
            e.get_processed_str(OutputOptions::FULL_TEXT)
        );
    }
    for s in [
        "VieetjNam",
        "duow",
        "qwer",
        "getUserNameByIdentifier",
        "ass",
        "Tieengs",
        "khuow",
        "aloo",
        "😀",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ] {
        let mut e = engine("telex");
        let mode = if s == "getUserNameByIdentifier" { Mode::English } else { Mode::Vietnamese };
        for c in s.chars() {
            e.process_key(c, mode);
        }
        println!(
            "PROBE\t{s}\t{}\t{}",
            e.output_str(),
            e.get_processed_str(OutputOptions::FULL_TEXT)
        );
    }
    println!("SIZE\t{}", std::mem::size_of::<Engine>());
    let mut e = engine("telex");
    e.process_str("tieengs", Mode::Vietnamese);
    e.remove_last_char(true);
    println!("BACKSPACE\ttieengs\t{}", e.output_str());
}
