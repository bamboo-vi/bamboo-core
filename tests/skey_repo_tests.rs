use bamboo_core::{Config, Engine, InputMethod, Mode};

fn type_word(engine: &mut Engine, input: &str) -> String {
    engine.reset();
    for ch in input.chars() {
        engine.process_key(ch, Mode::Vietnamese);
    }
    engine.output().to_string()
}

fn type_phrase(engine: &mut Engine, input: &str) -> String {
    engine.reset();
    let mut full = String::new();
    for ch in input.chars() {
        if ch == ' ' {
            full.push_str(&engine.output());
            full.push(' ');
            engine.reset();
        } else {
            engine.process_key(ch, Mode::Vietnamese);
        }
    }
    full.push_str(&engine.output());
    full
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. SKEY-ENGINE VSEQ PARITY (70 Vowel Sequences)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_skey_vseq_single_vowels() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "as"), "á");
    assert_eq!(type_word(&mut e, "af"), "à");
    assert_eq!(type_word(&mut e, "ar"), "ả");
    assert_eq!(type_word(&mut e, "ax"), "ã");
    assert_eq!(type_word(&mut e, "aj"), "ạ");
    assert_eq!(type_word(&mut e, "aas"), "ấ");
    assert_eq!(type_word(&mut e, "aws"), "ắ");
    assert_eq!(type_word(&mut e, "es"), "é");
    assert_eq!(type_word(&mut e, "ees"), "ế");
    assert_eq!(type_word(&mut e, "is"), "í");
    assert_eq!(type_word(&mut e, "os"), "ó");
    assert_eq!(type_word(&mut e, "oos"), "ố");
    assert_eq!(type_word(&mut e, "ows"), "ớ");
    assert_eq!(type_word(&mut e, "us"), "ú");
    assert_eq!(type_word(&mut e, "uws"), "ứ");
    assert_eq!(type_word(&mut e, "ys"), "ý");
}

#[test]
fn test_skey_vseq_open_diphthongs() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "ais"), "ái");
    assert_eq!(type_word(&mut e, "aos"), "áo");
    assert_eq!(type_word(&mut e, "aus"), "áu");
    assert_eq!(type_word(&mut e, "ays"), "áy");
    assert_eq!(type_word(&mut e, "aaus"), "ấu");
    assert_eq!(type_word(&mut e, "aays"), "ấy");
    assert_eq!(type_word(&mut e, "eos"), "éo");
    assert_eq!(type_word(&mut e, "eeus"), "ếu");
    assert_eq!(type_word(&mut e, "ias"), "ía");
    assert_eq!(type_word(&mut e, "ius"), "íu");
    assert_eq!(type_word(&mut e, "ois"), "ói");
    assert_eq!(type_word(&mut e, "oois"), "ối");
    assert_eq!(type_word(&mut e, "owis"), "ới");
    assert_eq!(type_word(&mut e, "uas"), "úa");
    assert_eq!(type_word(&mut e, "uis"), "úi");
    assert_eq!(type_word(&mut e, "uwas"), "ứa");
    assert_eq!(type_word(&mut e, "uwis"), "ứi");
    assert_eq!(type_word(&mut e, "uwus"), "ứu");
}

#[test]
fn test_skey_vseq_closed_diphthongs() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "tieengs"), "tiếng");
    assert_eq!(type_word(&mut e, "tieeps"), "tiếp");
    assert_eq!(type_word(&mut e, "toans"), "toán");
    assert_eq!(type_word(&mut e, "hoawcj"), "hoặc");
    assert_eq!(type_word(&mut e, "xuaans"), "xuấn");
    assert_eq!(type_word(&mut e, "thuees"), "thuế");
    assert_eq!(type_word(&mut e, "buoons"), "buốn");
    assert_eq!(type_word(&mut e, "cuowcs"), "cước");
    assert_eq!(type_word(&mut e, "khuyt"), "khuyt");
    assert_eq!(type_word(&mut e, "yeeng"), "yêng");
}

#[test]
fn test_skey_vseq_triphthongs() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "ieeus"), "iếu");
    assert_eq!(type_word(&mut e, "oais"), "oái");
    assert_eq!(type_word(&mut e, "oays"), "oáy");
    assert_eq!(type_word(&mut e, "oeos"), "oéo");
    assert_eq!(type_word(&mut e, "uoois"), "uối");
    assert_eq!(type_word(&mut e, "khuyas"), "khuýa");
    assert_eq!(type_word(&mut e, "khuyus"), "khuýu");
    assert_eq!(type_word(&mut e, "nguoiws"), "ngưới");
    assert_eq!(type_word(&mut e, "ruowus"), "rướu");
    assert_eq!(type_word(&mut e, "yeeus"), "yếu");
}

#[test]
fn test_skey_modern_vs_traditional_tone() {
    // Standard (New) style: hòa, khỏe
    let mut e_std = Engine::with_config(
        InputMethod::telex(),
        Config { std_tone_style: true, ..Default::default() },
    );
    assert_eq!(type_word(&mut e_std, "hoaf"), "hòa");
    assert_eq!(type_word(&mut e_std, "thowif"), "thời");
    assert_eq!(type_word(&mut e_std, "cuar"), "của");
    assert_eq!(type_word(&mut e_std, "khoer"), "khỏe");

    // Old style: hoà, khoẻ
    let mut e_old = Engine::with_config(
        InputMethod::telex(),
        Config { std_tone_style: false, ..Default::default() },
    );
    assert_eq!(type_word(&mut e_old, "hoaf"), "hoà");
    assert_eq!(type_word(&mut e_old, "oas"), "oá");
    assert_eq!(type_word(&mut e_old, "oes"), "oé");
    assert_eq!(type_word(&mut e_old, "khoer"), "khoẻ");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. SKEY-ENGINE TONE REPLACEMENT & REASSIGNMENT
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_skey_tone_replacement() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "saosf"), "sào"); // sáo + f -> sào
    assert_eq!(type_word(&mut e, "saojs"), "sáo"); // sạ + s -> sáo
    assert_eq!(type_word(&mut e, "saojr"), "sảo"); // sạo + r -> sảo
    assert_eq!(type_word(&mut e, "toansr"), "toản"); // toán + r -> toản
}

#[test]
fn test_skey_rapid_tone_reassignment() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "gisa"), "giá"); // gis -> gí + a -> giá
    assert_eq!(type_word(&mut e, "gifa"), "già"); // gif -> gì + a -> già
    assert_eq!(type_word(&mut e, "quisa"), "quía"); // qus -> qú + i + a -> quía
    assert_eq!(type_word(&mut e, "tofan"), "toàn"); // tof -> tò + a + n -> toàn
    assert_eq!(type_word(&mut e, "toafn"), "toàn"); // toa -> toa + f -> tòa + n -> toàn
    assert_eq!(type_word(&mut e, "hoafn"), "hoàn"); // hoaf -> hòa + n -> hoàn
}

#[test]
fn test_skey_rare_vowel_sequences() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_word(&mut e, "thuees"), "thuế");
    assert_eq!(type_word(&mut e, "hueef"), "huề");
    assert_eq!(type_word(&mut e, "hoawcj"), "hoặc");
    assert_eq!(type_word(&mut e, "thoawcs"), "thoắc");
    assert_eq!(type_word(&mut e, "thoawts"), "thoắt");
    assert_eq!(type_word(&mut e, "khuya"), "khuya");
    assert_eq!(type_word(&mut e, "khuyar"), "khuỷa");
    assert_eq!(type_word(&mut e, "ruowus"), "rướu");
    assert_eq!(type_word(&mut e, "huowuf"), "hườu");
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. SKEY-ENGINE ZERO-DELAY TYPING & RAPID STRESS STREAM
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_skey_zero_delay_stream_telex() {
    let mut e = Engine::new(InputMethod::telex());
    assert_eq!(type_phrase(&mut e, "xin chaof"), "xin chào");
    assert_eq!(type_phrase(&mut e, "tooi laf nguwowif Vieetj"), "tôi là người Việt");
    assert_eq!(type_phrase(&mut e, "hoanf toanf"), "hoàn toàn");
}

#[test]
fn test_skey_zero_delay_stream_vni() {
    let mut e = Engine::new(InputMethod::vni());
    assert_eq!(type_phrase(&mut e, "tie61ng Vie65t"), "tiếng Việt");
    assert_eq!(type_phrase(&mut e, "tru7o7ng2"), "trường");
    assert_eq!(type_phrase(&mut e, "viet65 nam"), "việt nam");
}
