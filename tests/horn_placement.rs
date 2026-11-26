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
    // After `h kh qu th`, `uow` types the Telex diphthong step `uơ` (as in
    // `thuowng` -> `thương`): the horn lands on the nearest vowel `o`, and a following
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

#[test]
fn telex_o_turns_ươ_back_into_uô() {
    // `o` after a two-horn `ươ` swaps it for `uô`, the same as after a
    // directly typed `ưo`. Before, the key undid the horn on `o` and the
    // word fell back to raw keys (`ruowifo`).
    check(
        InputMethod::telex(),
        &[
            ("ruowo", "ruô"),
            ("ruowifo", "ruồi"),
            ("ruowfio", "ruồi"),
            ("muownso", "muốn"),
            ("chuowngo", "chuông"),
            ("tuowngo", "tuông"),
        ],
    );
}

#[test]
fn telex_w_turns_uô_into_ươ() {
    // The reverse swap already worked; keep it that way.
    check(
        InputMethod::telex(),
        &[("ruooixw", "rưỡi"), ("tuoongw", "tương"), ("muoonsw", "mướn"), ("huowo", "huô")],
    );
}

#[test]
fn vni_6_turns_ươ_back_into_uô() {
    check(
        InputMethod::vni(),
        &[
            ("ruo76", "ruô"),
            ("ruo7i26", "ruồi"),
            ("ruo72i6", "ruồi"),
            ("muo7n16", "muốn"),
            ("chuo7ng6", "chuông"),
            ("tuo7ng6", "tuông"),
        ],
    );
}

#[test]
fn vni_7_turns_uô_into_ươ() {
    check(
        InputMethod::vni(),
        &[("ruo6i47", "rưỡi"), ("tuo6ng7", "tương"), ("muo6n17", "mướn"), ("huo76", "huô")],
    );
}
