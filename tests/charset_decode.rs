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
            let back = decode_charset(charset, &encode_charset(charset, &letter)).into_owned();
            match shared.get(&c) {
                None => assert_eq!(back, letter, "{charset}"),
                Some(group) => {
                    let back = back.chars().next().unwrap();
                    assert!(group.contains(&back), "{charset}: {c} decoded to {back}");
                    if group.iter().any(|g| g.is_lowercase()) {
                        assert!(back.is_lowercase(), "{charset}: {c} decoded to {back}");
                    }
                }
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
