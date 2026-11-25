use bamboo_core::{Engine, InputMethod, Mode};

fn check(im: InputMethod, cases: &[(&str, &str)]) {
    for (keys, expected) in cases {
        let mut engine = Engine::new(im.clone());
        engine.process_str(keys, Mode::Vietnamese);
        assert_eq!(engine.output(), *expected, "keys: {keys}");
    }
}

#[test]
fn telex_horn_on_o_when_nothing_follows_uo() {
    // `uow` types the documented Telex diphthong step `uơ` (as in `truowng`
    // -> `trương`): the horn lands on the nearest vowel `o`, and a following
    // coda still resolves it (`uowng` -> `ương`). A mid-word preedit is not
    // judged by dictionary rimes (`uơ` is no standalone word, but neither is
    // any other unfinished preedit); what matters is that completed words
    // come out right (`truowcs` -> `trước`, see the vi/uvie suite).
    check(InputMethod::telex(), &[("khuow", "khuơ"), ("thuow", "thuơ"), ("huow", "huơ")]);
}

#[test]
fn uow_spreads_both_horns_after_c_k_and_no_onset() {
    // `uơ` only ends real words after `h kh qu th` (`thuở`, `huơ`, `khuơ`); after `c`, `k` or
    // no onset it is always the first step of `ươ` (`cước`, `ước`), so both horns go on at once
    // instead of changing the shown text at the next key.
    check(InputMethod::telex(), &[("cuow", "cươ"), ("kuow", "kươ"), ("uow", "ươ"), ("uowr", "ưở")]);
    check(InputMethod::vni(), &[("cuo7", "cươ"), ("uo7", "ươ"), ("uo73", "ưở")]);
}

#[test]
fn telex_uow_spreads_both_horns_for_onsets_that_forbid_uơ() {
    // The rime `uơ` stands only after the `h kh qu th` onset class, so `khuơ`
    // keeps one horn, while `duơ` is incomplete and spreads the second horn
    // (`dươ`) — matching Go bamboo-core.
    check(
        InputMethod::telex(),
        &[("duow", "dươ"), ("dduow", "đươ"), ("tuow", "tươ"), ("nguow", "ngươ")],
    );
}

#[test]
fn telex_tone_lands_on_ơ_after_both_horns() {
    // Go's findToneTarget prefers the vowel carrying ơ/ê for the tone, coda
    // or not: `tuowf` is `tườ`, not `từơ` (bench-fcitx5 report #578/#579).
    check(InputMethod::telex(), &[("tuowf", "tườ"), ("tuowfj", "tượ")]);
}

#[test]
fn telex_horn_spreads_to_u_when_letter_follows() {
    check(InputMethod::telex(), &[("huouw", "hươu"), ("thuoiwx", "thưỡi")]);
}

#[test]
fn telex_uow_shortcut_still_works() {
    check(
        InputMethod::telex(),
        &[("huwowu", "hươu"), ("truwowjt", "trượt"), ("dduowngf", "đường"), ("nguwowif", "người")],
    );
}

#[test]
fn vni_horn_placement_after_uo() {
    check(InputMethod::vni(), &[("khuo7", "khuơ"), ("huou7", "hươu")]);
}
