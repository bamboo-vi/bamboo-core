use bamboo_core::{Config, Engine, InputMethod, Mode, OutputOptions};

fn engine(input_method: InputMethod) -> Engine {
    Engine::with_config(input_method, Config::default())
}

#[test]
fn refused_vni_shape_key_keeps_the_composed_word() {
    for input_method in
        [InputMethod::vni(), InputMethod::telex_vni(), InputMethod::telex_vni_viqr()]
    {
        for (key, expected) in [('6', "hạo6"), ('9', "hạo9")] {
            let mut e = engine(input_method.clone());
            e.process_str("hao12345", Mode::Vietnamese);
            assert_eq!(e.output(), "hạo");
            assert!(e.is_valid_word());

            e.process_key(key, Mode::Vietnamese);
            assert_eq!(e.output(), expected, "preset {:?}, key {key}", input_method);

            e.process_key('x', Mode::Vietnamese);
            assert_eq!(e.output(), format!("{expected}x"));
        }
    }
}

#[test]
fn refused_telex_tone_key_keeps_the_composed_word() {
    let mut e = engine(InputMethod::telex());
    e.process_str("vieejt", Mode::Vietnamese);
    assert_eq!(e.output(), "việt");

    e.process_key('f', Mode::Vietnamese);
    assert_eq!(e.output(), "việtf");
    e.process_key('x', Mode::Vietnamese);
    assert_eq!(e.output(), "việtfx");
}

#[test]
fn backspace_restores_the_composition_and_bypass_state() {
    let mut e = engine(InputMethod::vni());
    e.process_str("hao12345", Mode::Vietnamese);
    e.process_key('6', Mode::Vietnamese);
    e.process_key('x', Mode::Vietnamese);
    assert_eq!(e.output(), "hạo6x");

    e.undo_last_key();
    assert_eq!(e.output(), "hạo6");
    e.undo_last_key();
    assert_eq!(e.output(), "hạo");
}

#[test]
fn ordinary_english_fallback_still_restores_raw_keys() {
    let mut e = engine(InputMethod::telex());
    e.process_str("cash", Mode::Vietnamese);
    assert_eq!(e.output(), "cash");

    e.reset();
    e.process_str("street", Mode::Vietnamese);
    assert_eq!(e.output(), "street");
}

#[test]
fn refused_telex_letter_that_is_not_a_tone_key_restores_english() {
    // Letters are both letters and mark keys in Telex, so a letter refused
    // after a valid syllable must not keep the composition.
    for input in [
        "are",
        "good",
        "system",
        "information",
        "order",
        "need",
        "case",
        "before",
        "research",
        "management",
        "meter",
    ] {
        let mut e = engine(InputMethod::telex());
        e.process_str(input, Mode::Vietnamese);
        assert_eq!(e.output(), input);
    }
}

#[test]
fn tone_cycling_is_not_a_refused_mark() {
    // A tone key that applies (cycling) must win over the refuse-preserve rule.
    let mut e = engine(InputMethod::telex());
    e.process_str("nghieengsf", Mode::Vietnamese);
    assert_eq!(e.output(), "nghiềng");

    e.process_key(' ', Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "nghiềng ");

    let mut e = engine(InputMethod::telex());
    e.process_str("nghieengsr", Mode::Vietnamese);
    assert_eq!(e.output(), "nghiểng");
    e.process_key('f', Mode::Vietnamese);
    assert_eq!(e.output(), "nghiềng");
}

#[test]
fn custom_tone_key_follows_the_keymap() {
    // The preserve rule must derive from the input method's tone keys, not
    // from a hardcoded Telex key set: huyền moved from `f` to `q` here.
    let im = InputMethod::from_definition(
        "Custom",
        [
            ("z", "XoaDauThanh"),
            ("s", "DauSac"),
            ("q", "DauHuyen"),
            ("r", "DauHoi"),
            ("x", "DauNga"),
            ("j", "DauNang"),
            ("a", "A_Â"),
            ("e", "E_Ê"),
            ("o", "O_Ô"),
            ("w", "UOAÔ_ƯƠĂƠ"),
            ("d", "D_Đ"),
        ],
    );
    let mut e = engine(im);
    e.process_str("vieejt", Mode::Vietnamese);
    e.process_key('q', Mode::Vietnamese);
    assert_eq!(e.output(), "việtq");
}

#[test]
fn english_vietnamese_switch_and_bypass_reset_keep_working() {
    let mut e = engine(InputMethod::telex());
    e.process_str("hello ", Mode::English);
    e.process_str("vieejts", Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "hello viết");

    let mut e = engine(InputMethod::telex());
    e.process_str("vieejt", Mode::Vietnamese);
    e.process_key('f', Mode::Vietnamese);
    e.process_key('x', Mode::Vietnamese);
    e.process_key(' ', Mode::Vietnamese);
    e.process_str("vieejts", Mode::Vietnamese);
    assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "việtfx viết");
}
