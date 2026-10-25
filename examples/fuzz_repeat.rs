use bamboo_core::{Config, Engine, InputMethod, Mode};
fn main() {
    let words = [
        "tieengs",
        "vieetj",
        "huowng",
        "quoocs",
        "nguwowif",
        "namf",
        "khongf",
        "duowcj",
        "nhuwngf",
        "moiw",
        "laaj",
        "tuoif",
        "troiwf",
        "doocs",
        "muaws",
        "hoas",
        "chuyeenn",
        "thuyeet",
        "truwowjt",
        "nghieengs",
    ];
    for (label, cfg) in [
        ("repeat-ON", Config::default()),
        ("repeat-OFF", Config::builder().auto_correct(false).build()),
    ] {
        let mut e = Engine::with_config(InputMethod::telex(), cfg);
        let mut keys = 0;
        for i in 0..50_000 {
            let w = words[i % words.len()];
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        println!(
            "{} 50kw repeat-20words: keys={} states={} arena={} flat={} map={} used={}B alloc={}B text={}B",
            label,
            keys,
            e.dfa_state_count(),
            e.dfa_arena_len(),
            e.dfa_flat_len(),
            e.dfa_composition_count(),
            e.dfa_memory_used(),
            e.dfa_memory_allocated(),
            e.committed_text_len()
        );
    }
}
