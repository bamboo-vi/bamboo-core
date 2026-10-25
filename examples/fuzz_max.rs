//! Strong fuzz to find DFA worst-case ceiling (5-10h typing).
//! Run: cargo run --release --example fuzz_max [-- <N>]
//! Measures states/arena/flat/map len+cap, used/alloc, text growth.

use bamboo_core::{Config, Engine, InputMethod, Mode};

fn fmt_b(b: usize) -> String {
    if b < 1024 {
        format!("{}B", b)
    } else if b < 1024 * 1024 {
        format!("{:.1}KB", b as f64 / 1024.0)
    } else {
        format!("{:.2}MB", b as f64 / 1024.0 / 1024.0)
    }
}

fn report(tag: &str, e: &Engine, keys: usize) {
    let states = e.dfa_state_count();
    let arena = e.dfa_arena_len();
    let flat = e.dfa_flat_len();
    let map = e.dfa_composition_count();
    let used = e.dfa_memory_used();
    let alloc = e.dfa_memory_allocated();
    println!(
        "{:<28} keys={:<8} states={:<7} (cap {}) arena={:<7} (cap {}) flat={:<7} (cap {}) map={:<6} (cap {}) used={:<8} alloc={:<8} text={}/cap{} bypass={}",
        tag,
        keys,
        states,
        e.dfa_states_capacity(),
        arena,
        e.dfa_arena_capacity(),
        flat,
        e.dfa_flat_capacity(),
        map,
        e.dfa_map_capacity(),
        fmt_b(used),
        fmt_b(alloc),
        fmt_b(e.committed_text_len()),
        fmt_b(e.committed_text_capacity()),
        e.bypass_allocated(),
    );
}

/// deterministic unique invalid word, ascii-only, len<=12, base36-ish
fn unique_word(i: usize) -> String {
    // prefix "qzx" (invalid cluster) + base26 suffix + digit suffix to force uniqueness
    // e.g. qzxbqj... + number. Keep len 8..12.
    let mut x = i;
    let mut s = String::from("qzx");
    const ALPH: &[u8] = b"bcdfghjklmnpqrstvwxzq"; // 20 letters, mostly invalid clusters
    for _ in 0..5 {
        s.push(ALPH[x % ALPH.len()] as char);
        x /= ALPH.len();
        if x == 0 {
            break;
        }
    }
    // append decimal digits to guarantee uniqueness across large N
    s.push_str(&format!("{:05}", i % 100000));
    // truncate to 12 to stay in DFA path (MAX_ACTIVE_TRANS=16 minus space)
    s.truncate(12);
    s
}

fn unique_english(i: usize) -> String {
    // code-like nonsense: fn_var_<i>_x
    format!("fn_var_{:05}_qx", i % 100000)
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(60_000);
    println!("=== FUZZ MAX N={} unique words/scenario ===", n);
    println!("State=88B Trans=16B. used=states*88+arena*16+flat+map*16. alloc=cap-based.");
    println!();

    // ---- 0. commit semantics check ----
    {
        let mut e = Engine::new(InputMethod::telex()); // default auto_correct=true
        let s0 = e.dfa_state_count();
        e.process_str("qzxbcq", Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();
        let s1 = e.dfa_state_count();
        println!(
            "[commit-semantics] auto_correct=ON(default) invalid 'qzxbcq'+space+commit: states {} -> {} (delta {})",
            s0,
            s1,
            s1 as isize - s0 as isize
        );

        let cfg_off = Config::builder().auto_correct(false).build();
        let mut e2 = Engine::with_config(InputMethod::telex(), cfg_off);
        let s0 = e2.dfa_state_count();
        e2.process_str("qzxbcq", Mode::Vietnamese);
        e2.process_key(' ', Mode::Vietnamese);
        e2.commit();
        let s1 = e2.dfa_state_count();
        println!(
            "[commit-semantics] auto_correct=OFF invalid 'qzxbcq'+space+commit: states {} -> {} (delta {})",
            s0,
            s1,
            s1 as isize - s0 as isize
        );

        let mut e3 = Engine::new(InputMethod::telex());
        let s0 = e3.dfa_state_count();
        e3.process_str("hello_world", Mode::English);
        e3.process_key(' ', Mode::English);
        e3.commit();
        let s1 = e3.dfa_state_count();
        println!(
            "[commit-semantics] Mode::English 'hello_world': states {} -> {} (delta {})",
            s0,
            s1,
            s1 as isize - s0 as isize
        );
        println!();
    }

    // ---- 1. Valid saturation (Telex fc*vowel*tone) ----
    {
        let mut e = Engine::new(InputMethod::telex());
        let fc = [
            "", "b", "c", "ch", "d", "dd", "g", "gh", "h", "k", "kh", "l", "m", "n", "nh", "ng",
            "ngh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
        ];
        let vowels = [
            "a", "e", "i", "o", "u", "y", "aa", "ee", "oo", "aw", "ow", "uw", "ai", "ao", "au",
            "ay", "ie", "oa", "oe", "oi", "ua", "ue", "ui", "uo", "uy",
        ];
        let tones = ["", "s", "f", "r", "x", "j"];
        let mut keys = 0;
        let mut count = 0;
        for f in &fc {
            for v in &vowels {
                for t in &tones {
                    let w = format!("{}{}{}", f, v, t);
                    for c in w.chars() {
                        e.process_key(c, Mode::Vietnamese);
                        keys += 1;
                    }
                    e.process_key(' ', Mode::Vietnamese);
                    keys += 1;
                    e.commit();
                    count += 1;
                }
            }
        }
        report(&format!("1.valid-telex {}words", count), &e, keys);
    }

    // ---- 2. WORST: unique invalid, auto_correct OFF, Vietnamese mode ----
    {
        let cfg_off = Config::builder().auto_correct(false).build();
        let mut e = Engine::with_config(InputMethod::telex(), cfg_off);
        let mut keys = 0;
        let checkpoints = [1000, 5000, 10_000, 20_000, 30_000, 50_000];
        // we run up to n; print at checkpoints <= n
        for i in 0..n {
            let w = unique_word(i);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
            // commit every word to mimic real typing (space already commits, extra commit is no-op)
            if i % 5 == 0 {
                e.commit();
            }
            if checkpoints.contains(&(i + 1)) {
                report(&format!("2.worst-OFF-telex-vi {}w", i + 1), &e, keys);
            }
        }
        if !checkpoints.contains(&n) {
            report(&format!("2.worst-OFF-telex-vi {}w FINAL", n), &e, keys);
        }
        // rate
        let states = e.dfa_state_count();
        println!(
            "    -> rate worst-OFF: {:.3} states/key, {:.2} states/word",
            states as f64 / keys as f64,
            states as f64 / n as f64
        );
        println!();
    }

    // ---- 3. Same unique invalid but auto_correct ON ----
    {
        let mut e = Engine::new(InputMethod::telex());
        let mut keys = 0;
        for i in 0..n {
            let w = unique_word(i);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
            if i % 5 == 0 {
                e.commit();
            }
            if [1000, 5000, 10_000, 20_000, 30_000, 50_000].contains(&(i + 1)) {
                report(&format!("3.invalid-ON-telex-vi {}w", i + 1), &e, keys);
            }
        }
        if ![1000, 5000, 10_000, 20_000, 30_000, 50_000].contains(&n) {
            report(&format!("3.invalid-ON-telex-vi {}w FINAL", n), &e, keys);
        }
        let states = e.dfa_state_count();
        println!("    -> rate invalid-ON: {:.4} states/key", states as f64 / keys as f64);
        println!();
    }

    // ---- 4. English nonsense in Vietnamese mode, OFF vs ON + English mode ----
    {
        // 4a: Vietnamese mode + OFF (worst english-as-vietnamese)
        let cfg_off = Config::builder().auto_correct(false).build();
        let mut e = Engine::with_config(InputMethod::telex(), cfg_off);
        let mut keys = 0;
        let m = (n / 2).max(5000);
        for i in 0..m {
            let w = unique_english(i);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        report(&format!("4a.english-in-VNmode-OFF {}w", m), &e, keys);

        // 4b: Vietnamese mode + ON
        let mut e = Engine::new(InputMethod::telex());
        let mut keys = 0;
        for i in 0..m {
            let w = unique_english(i);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        report(&format!("4b.english-in-VNmode-ON {}w", m), &e, keys);

        // 4c: English mode (should be 0 growth)
        let mut e = Engine::new(InputMethod::telex());
        let mut keys = 0;
        for i in 0..m {
            let w = unique_english(i);
            for c in w.chars() {
                e.process_key(c, Mode::English);
                keys += 1;
            }
            e.process_key(' ', Mode::English);
            keys += 1;
        }
        e.commit();
        report(&format!("4c.english-in-ENmode {}w", m), &e, keys);
        println!();
    }

    // ---- 5. Mixed realistic: valid + invalid alternating, ON ----
    {
        let mut e = Engine::new(InputMethod::telex());
        let valid = ["tieengs", "vieetj", "huowng", "khongf", "duowcj", "nghieengs"];
        let mut keys = 0;
        for i in 0..n {
            let w = if i % 2 == 0 { valid[i % valid.len()].to_string() } else { unique_word(i) };
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        report(&format!("5.mixed-ON {}w", n), &e, keys);
    }

    // ---- 6. VNI worst-OFF (digits) quick check ----
    {
        let cfg_off = Config::builder().auto_correct(false).build();
        let mut e = Engine::with_config(InputMethod::vni(), cfg_off);
        let mut keys = 0;
        let m = (n / 3).max(5000);
        for i in 0..m {
            let w = format!("a{:05}e{:02}", i % 100000, i % 100);
            for c in w.chars() {
                e.process_key(c, Mode::Vietnamese);
                keys += 1;
            }
            e.process_key(' ', Mode::Vietnamese);
            keys += 1;
        }
        e.commit();
        report(&format!("6.vni-OFF {}w", m), &e, keys);
    }

    // ---- 7. Sustained no-commit single buffer (cap check) ----
    {
        let mut e = Engine::new(InputMethod::telex());
        let long: String = (0..500).map(unique_word).collect::<Vec<_>>().join("");
        let mut keys = 0;
        for c in long.chars() {
            e.process_key(c, Mode::Vietnamese);
            keys += 1;
            if keys >= 2000 {
                break;
            }
        }
        report("7.sustained-no-commit 2000keys", &e, keys);
        println!(
            "    active_len={} snapshot_len={} (caps 16 each)",
            e.active_len(),
            e.snapshot_len()
        );
    }

    println!();
    println!(
        "Done. Extrapolate: 5h@5keys/s=90k keys, 5h@10keys/s=180k keys, 10h@10keys/s=360k keys."
    );
}
