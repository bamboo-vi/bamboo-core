use bamboo_core::{Config, Engine, InputMethod, Mode, ToneStyle};

fn typed(style: ToneStyle, keys: &str) -> String {
    typed_with(InputMethod::telex(), style, keys)
}

fn typed_with(im: InputMethod, style: ToneStyle, keys: &str) -> String {
    let config = Config::builder().tone_style(style).build();
    let mut engine = Engine::with_config(im, config);
    for key in keys.chars() {
        engine.process_key(key, Mode::Vietnamese);
    }
    engine.output().into_owned()
}

// ToneStyle::Traditional gives hoà, ToneStyle::Modern gives hòa; only oa, oe and uy differ.
// A vowel with a circumflex or horn keeps the tone in both, before the coda is typed too.
#[test]
fn hoa_style_puts_the_tone_on_the_marked_second_vowel() {
    for (keys, want) in
        [("tiees", "tiế"), ("vieej", "việ"), ("chuaar", "chuẩ"), ("khuyeens", "khuyến")]
    {
        assert_eq!(typed(ToneStyle::Traditional, keys), want, "{keys}");
        assert_eq!(typed(ToneStyle::Modern, keys), want, "{keys}");
    }
}

#[test]
fn hoa_style_still_marks_oa_on_the_second_vowel() {
    assert_eq!(typed(ToneStyle::Traditional, "hoaf"), "hoà");
    assert_eq!(typed(ToneStyle::Modern, "hoaf"), "hòa");
    assert_eq!(typed(ToneStyle::Traditional, "tias"), "tía");
    assert_eq!(typed(ToneStyle::Traditional, "cuar"), "của");
}

#[test]
fn vni_hoa_style_puts_the_tone_on_the_marked_second_vowel() {
    for (keys, want) in
        [("tie61", "tiế"), ("vie65", "việ"), ("chua63", "chuẩ"), ("khuye6n1", "khuyến")]
    {
        assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Traditional, keys), want, "{keys}");
        assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Modern, keys), want, "{keys}");
    }
}

#[test]
fn vni_hoa_style_still_marks_oa_on_the_second_vowel() {
    assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Traditional, "hoa2"), "hoà");
    assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Modern, "hoa2"), "hòa");
    assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Traditional, "tia1"), "tía");
    assert_eq!(typed_with(InputMethod::vni(), ToneStyle::Traditional, "cua3"), "của");
}
