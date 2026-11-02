use bamboo_core::{Config, Engine, InputMethod, Mode};

fn typed(keys: &str) -> String {
    let config = Config::builder().auto_correct(true).build();
    let mut engine = Engine::with_config(InputMethod::telex(), config);
    for key in keys.chars() {
        engine.process_key(key, Mode::Vietnamese);
    }
    engine.output().into_owned()
}

#[test]
fn auto_correct_keeps_consonant_abbreviations_with_dd() {
    assert_eq!(typed("ddc"), "đc");
    assert_eq!(typed("ddt"), "đt");
    assert_eq!(typed("cdd"), "cđ");
    assert_eq!(typed("vcdd"), "vcđ");
    assert_eq!(typed("bcdd"), "bcđ");
}

#[test]
fn auto_correct_still_restores_other_consonant_clusters() {
    assert_eq!(typed("tooltip"), "tooltip");
    assert_eq!(typed("rebooot"), "rebooot");
}
