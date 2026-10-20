use bamboo_core::{Config, Engine, InputMethod, Mode};

#[test]
fn camelcase_word_depends_on_auto_correct_option() {
    let mut default_engine = Engine::new(InputMethod::telex());
    default_engine.process_str("VieetjNam", Mode::Vietnamese);
    assert_eq!(default_engine.output(), "VieetjNam");

    let config = Config::builder().auto_correct(false).build();
    let mut no_restore = Engine::with_config(InputMethod::telex(), config);
    no_restore.process_str("VieetjNam", Mode::Vietnamese);
    assert_eq!(no_restore.output(), "ViệtNam");
}
