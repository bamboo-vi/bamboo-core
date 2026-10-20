//! Read a TSV event stream produced by bench-fcitx5/compare_corpora.py.
use bamboo_core::{Config, Engine, InputMethodPreset, Mode, OutputOptions};
use std::io::{self, BufRead};

fn engine(im: &str) -> Engine {
    let preset = match im {
        "vni" => InputMethodPreset::Vni,
        "viqr" => InputMethodPreset::Viqr,
        _ => InputMethodPreset::Telex,
    };
    if std::env::var_os("BAMBOO_AUTO_CORRECT_OFF").is_some() {
        Engine::from_preset_with_config(preset, Config::builder().auto_correct(false).build())
    } else {
        Engine::from_preset(preset)
    }
}

fn emit(s: &str) {
    for b in s.as_bytes() {
        print!("{b:02x}");
    }
    println!();
}

fn main() {
    let mut e = engine("telex");
    for line in io::stdin().lock().lines() {
        let line = line.expect("read event");
        let mut fields = line.split('\t');
        let op = fields.next().unwrap_or("");
        let arg = fields.next().unwrap_or("");
        let mode = fields.next().unwrap_or("");
        match op {
            "set_im" => e = engine(arg),
            "key" => {
                let scalar = u32::from_str_radix(arg, 16).expect("key scalar");
                let key = char::from_u32(scalar).expect("valid key");
                e.process_key(key, if mode == "en" { Mode::English } else { Mode::Vietnamese });
                emit(&e.get_processed_str_cow(OutputOptions::NONE));
            }
            "backspace" => {
                if arg == "grapheme" {
                    e.remove_last_output_char();
                } else {
                    e.undo_last_key();
                }
                emit(&e.get_processed_str_cow(OutputOptions::NONE));
            }
            "commit" => {
                emit(&e.get_processed_str_cow(OutputOptions::NONE));
                e.reset();
            }
            "focus_reset" => e.reset(),
            "set_mode" => emit(&e.get_processed_str_cow(OutputOptions::NONE)),
            "rebuild" => {}
            _ => panic!("unknown operation: {op}"),
        }
    }
}
