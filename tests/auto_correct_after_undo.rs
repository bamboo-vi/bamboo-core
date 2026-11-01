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
fn restoring_an_undone_mark_drops_the_undo_key() {
    assert_eq!(typed("toool"), "tool");
    assert_eq!(typed("toooltip"), "tooltip");
}

#[test]
fn restoring_an_undone_tone_drops_the_undo_key() {
    assert_eq!(typed("assn"), "asn");
}

#[test]
fn restoring_without_undo_keeps_every_key() {
    assert_eq!(typed("tooltip"), "tooltip");
}
