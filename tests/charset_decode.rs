use std::borrow::Cow;
use std::collections::HashMap;

use bamboo_core::advanced::{charset_names, get_charset_definition};
use bamboo_core::{Charset, decode_charset, encode_charset};

// No vowel is followed by `.`, `?` or `'`, which VIQR reads as tone marks.
const TEXT: &str = "Đức Hưởng ăn quýt, Ỷ Lại uống trà ở Phố Hiến; ĐÊM ẤY GIÓ BẤC LỘNG, \
    trời thì ẩm ướt, ngoài sân cỏ úa dưới mưa phùn. Quỳnh Chiêu nhặt lá ổi ở bờ ao, \
    Ơn Ngữ xoè tay hứng: Ồ, Ừ, Ặc, Ẵm, Ẳng, Ẫy, Ẩn, Ậy, Ẽo, Ẻ, Ễ, Ệ, Ĩ, Ị, Ỏ, Ọ, Ổ, Ỗ, Ộ, \
    Ớ, Ờ, Ở, Ỡ, Ợ, Ủ, Ũ, Ụ, Ứ, Ữ, Ự, Ỳ, Ỵ, Ỹ, Ắ, Ằ, Ầ, Ấ, Ẹ, Ề, Ế, Ể, Ò, Ó, Õ, È, É, \
    Ì, Í, Ù, Ú, Ý, À, Á, Â, Ã, Ê, Ô, Ư, Ơ, Ă, Đ";

fn legacy_charsets() -> Vec<Charset> {
    charset_names()
        .filter_map(|name| name.parse().ok())
        .filter(|&charset| charset != Charset::Unicode)
        .collect()
}

/// Groups of letters a charset encodes to the same code.
fn shared_codes(charset: Charset) -> HashMap<char, Vec<char>> {
    let def = get_charset_definition(charset.as_str()).expect("legacy charset has a table");
    let mut by_code: HashMap<&str, Vec<char>> = HashMap::new();
    for (&c, &code) in def.entries() {
        by_code.entry(code).or_default().push(c);
    }
    by_code
        .into_values()
        .filter(|group| group.len() > 1)
        .flat_map(|group| group.clone().into_iter().map(move |c| (c, group.clone())))
        .collect()
}

#[test]
fn every_letter_decodes_back() {
    for charset in legacy_charsets() {
        let def = get_charset_definition(charset.as_str()).unwrap();
        let shared = shared_codes(charset);
        for &c in def.keys() {
            let letter = c.to_string();
            let encoded = encode_charset(charset, &letter);
            let back = decode_charset(charset, &encoded).into_owned();
            match shared.get(&c) {
                None => assert_eq!(back, letter, "{charset}"),
                Some(group) if group.iter().all(|g| g.to_lowercase().eq(c.to_lowercase())) => {
                    assert_eq!(back, c.to_lowercase().to_string(), "{charset}");
                }
                Some(_) => assert_eq!(back, encoded, "{charset}: {c} must stay undecoded"),
            }
        }
    }
}

#[test]
fn text_decodes_back() {
    for charset in legacy_charsets() {
        let lower = TEXT.to_lowercase();
        let encoded = encode_charset(charset, &lower);
        assert_eq!(decode_charset(charset, &encoded), lower, "{charset}");

        if shared_codes(charset).is_empty() {
            let encoded = encode_charset(charset, TEXT);
            assert_eq!(decode_charset(charset, &encoded), TEXT, "{charset}");
        }
    }
}

#[test]
fn shared_uppercase_codes_decode_to_lowercase() {
    // TCVN3 showed these capitals with a separate font, which copying the text loses.
    let encoded = encode_charset(Charset::Tcvn3, "TIẾNG Việt");
    assert_eq!(decode_charset(Charset::Tcvn3, &encoded), "TIếNG Việt");

    for charset in [Charset::Tcvn3, Charset::VietwareFull, Charset::Vps, Charset::Viscii] {
        let def = get_charset_definition(charset.as_str()).unwrap();
        let expected: String = TEXT
            .chars()
            .map(|c| {
                let lower = c.to_lowercase().next().unwrap();
                if lower != c && def.get(&c).is_some() && def.get(&c) == def.get(&lower) {
                    lower
                } else {
                    c
                }
            })
            .collect();
        let encoded = encode_charset(charset, TEXT);
        assert_eq!(decode_charset(charset, &encoded), expected, "{charset}");
    }
}

#[test]
fn documented_limits() {
    // BKHCM 1 gives Ặ and Ụ one code; guessing would silently change the letter.
    let encoded = encode_charset(Charset::Bkhcm1, "Ụ");
    assert_eq!(decode_charset(Charset::Bkhcm1, &encoded), encoded);

    // BKHCM 1 spells some capitals with ASCII punctuation.
    assert_eq!(decode_charset(Charset::Bkhcm1, "{a}"), "ÝaĐ");

    // VIQR reads a full stop after a vowel as a dot below.
    assert_eq!(decode_charset(Charset::Viqr, "mu+a."), "mưạ");

    // Full NFD is not the Unicode tổ hợp form.
    let nfd = "e\u{302}\u{301}";
    assert_eq!(decode_charset(Charset::UnicodeCompound, nfd), nfd);
    assert_eq!(decode_charset(Charset::UnicodeCompound, "\u{ea}\u{301}"), "ế");
}

#[test]
fn decodes_viqr_escapes() {
    // UniKey's VIQR output for "mưa.", "Hà Nội?" and "a\b".
    assert_eq!(decode_charset(Charset::Viqr, "mu+a\\."), "mưa.");
    assert_eq!(decode_charset(Charset::Viqr, "Ha` No^.i\\?"), "Hà Nội?");
    assert_eq!(decode_charset(Charset::Viqr, "a\\\\b"), "a\\b");
    // A backslash before anything else is not an escape.
    assert_eq!(decode_charset(Charset::Viqr, "C:\\new"), "C:\\new");
}

#[test]
fn decodes_viscii_capitals_on_control_codes() {
    // Bytes 02 05 06 A0 14 19 1E, as glibc iconv writes these capitals in VISCII.
    let viscii = "\u{2}\u{5}\u{6}\u{a0}\u{14}\u{19}\u{1e}";
    assert_eq!(decode_charset(Charset::Viscii, viscii), "ẲẴẪÕỶỸỴ");
    assert_eq!(decode_charset(Charset::Tcvn3, viscii), viscii);
}

#[test]
fn decodes_known_legacy_text() {
    assert_eq!(decode_charset(Charset::VniWindows, "tieáng Vieät"), "tiếng Việt");
    assert_eq!(decode_charset(Charset::Viqr, "tie^'ng Vie^.t"), "tiếng Việt");
    assert_eq!(decode_charset(Charset::NcrDecimal, "Vi&#7879;t"), "Việt");
}

#[test]
fn text_without_codes_is_borrowed() {
    let plain = decode_charset(Charset::Tcvn3, "hello world");
    assert!(matches!(plain, Cow::Borrowed("hello world")));

    let unicode = decode_charset(Charset::Unicode, "tiếng Việt");
    assert!(matches!(unicode, Cow::Borrowed("tiếng Việt")));
}
