//! Vietnamese spelling validation logic based on CVC (Consonant-Vowel-Consonant) structure.

// Static token representation: (chars, length)
type Token = ([char; 4], u8);

/// Strips diacritic marks from a character (mark 0 = canonical base).
/// Used in the innermost token-match loop — one decode-table load.
/// Only the 7 bare marked chars strip; toned variants (e.g. 'ầ') and
/// everything else are identity.
#[inline(always)]
const fn strip_mark(c: char) -> char {
    match crate::phonetics::mark_root(c) {
        Some(root) => root,
        None => c,
    }
}

// Any known onset may pair with any rime; only the rime itself is constrained.
static FC: &[Token] = &[
    (['b', '\0', '\0', '\0'], 1),
    (['c', '\0', '\0', '\0'], 1),
    (['c', 'h', '\0', '\0'], 2),
    (['d', '\0', '\0', '\0'], 1),
    (['đ', '\0', '\0', '\0'], 1),
    (['g', '\0', '\0', '\0'], 1),
    (['g', 'h', '\0', '\0'], 2),
    (['g', 'i', '\0', '\0'], 2),
    (['h', '\0', '\0', '\0'], 1),
    (['k', '\0', '\0', '\0'], 1),
    (['k', 'h', '\0', '\0'], 2),
    (['k', 'r', '\0', '\0'], 2),
    (['l', '\0', '\0', '\0'], 1),
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['n', 'g', '\0', '\0'], 2),
    (['n', 'g', 'h', '\0'], 3),
    (['n', 'h', '\0', '\0'], 2),
    (['p', '\0', '\0', '\0'], 1),
    (['p', 'h', '\0', '\0'], 2),
    (['q', 'u', '\0', '\0'], 2),
    (['r', '\0', '\0', '\0'], 1),
    (['s', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
    (['t', 'h', '\0', '\0'], 2),
    (['t', 'r', '\0', '\0'], 2),
    (['v', '\0', '\0', '\0'], 1),
    (['x', '\0', '\0', '\0'], 1),
    (['z', '\0', '\0', '\0'], 1),
];

static FC_ROWS: &[&[Token]] = &[FC];

static VO_0: &[Token] = &[
    (['ê', '\0', '\0', '\0'], 1),
    (['i', '\0', '\0', '\0'], 1),
    (['u', 'a', '\0', '\0'], 2),
    (['u', 'ê', '\0', '\0'], 2),
    (['u', 'y', '\0', '\0'], 2),
    (['y', '\0', '\0', '\0'], 1),
];
static VO_1: &[Token] = &[
    (['a', '\0', '\0', '\0'], 1),
    (['i', 'ê', '\0', '\0'], 2),
    (['o', 'a', '\0', '\0'], 2),
    (['u', 'y', 'ê', '\0'], 3),
    (['y', 'ê', '\0', '\0'], 2),
];
static VO_2: &[Token] = &[
    (['â', '\0', '\0', '\0'], 1),
    (['ă', '\0', '\0', '\0'], 1),
    (['e', '\0', '\0', '\0'], 1),
    (['o', '\0', '\0', '\0'], 1),
    (['o', 'o', '\0', '\0'], 2),
    (['ô', '\0', '\0', '\0'], 1),
    (['ơ', '\0', '\0', '\0'], 1),
    (['o', 'e', '\0', '\0'], 2),
    (['u', '\0', '\0', '\0'], 1),
    (['ư', '\0', '\0', '\0'], 1),
    (['u', 'â', '\0', '\0'], 2),
    (['u', 'ô', '\0', '\0'], 2),
    (['ư', 'ơ', '\0', '\0'], 2),
];
static VO_3: &[Token] = &[(['o', 'ă', '\0', '\0'], 2)];
static VO_4: &[Token] = &[(['u', 'ơ', '\0', '\0'], 2)];
static VO_5: &[Token] = &[
    (['a', 'i', '\0', '\0'], 2),
    (['a', 'o', '\0', '\0'], 2),
    (['a', 'u', '\0', '\0'], 2),
    (['â', 'u', '\0', '\0'], 2),
    (['a', 'y', '\0', '\0'], 2),
    (['â', 'y', '\0', '\0'], 2),
    (['e', 'o', '\0', '\0'], 2),
    (['ê', 'u', '\0', '\0'], 2),
    (['i', 'a', '\0', '\0'], 2),
    (['i', 'ê', 'u', '\0'], 3),
    (['i', 'u', '\0', '\0'], 2),
    (['o', 'a', 'i', '\0'], 3),
    (['o', 'a', 'o', '\0'], 3),
    (['o', 'a', 'y', '\0'], 3),
    (['o', 'e', 'o', '\0'], 3),
    (['o', 'i', '\0', '\0'], 2),
    (['ô', 'i', '\0', '\0'], 2),
    (['ơ', 'i', '\0', '\0'], 2),
    (['ư', 'a', '\0', '\0'], 2),
    (['u', 'â', 'y', '\0'], 3),
    (['u', 'i', '\0', '\0'], 2),
    (['ư', 'i', '\0', '\0'], 2),
    (['u', 'ô', 'i', '\0'], 3),
    (['ư', 'ơ', 'i', '\0'], 3),
    (['ư', 'ơ', 'u', '\0'], 3),
    (['ư', 'u', '\0', '\0'], 2),
    (['u', 'y', 'a', '\0'], 3),
    (['u', 'y', 'u', '\0'], 3),
    (['u', 'ê', 'u', '\0'], 3),
    (['y', 'ê', 'u', '\0'], 3),
];
static VO_6: &[Token] = &[(['ă', '\0', '\0', '\0'], 1), (['u', '\0', '\0', '\0'], 1)];
static VO_7: &[Token] = &[(['i', '\0', '\0', '\0'], 1)];

static VO_ROWS: &[&[Token]] = &[VO_0, VO_1, VO_2, VO_3, VO_4, VO_5, VO_6, VO_7];

static LC_0: &[Token] = &[(['c', 'h', '\0', '\0'], 2), (['n', 'h', '\0', '\0'], 2)];
static LC_1: &[Token] = &[(['c', '\0', '\0', '\0'], 1), (['n', 'g', '\0', '\0'], 2)];
static LC_2: &[Token] = &[
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['p', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
];
static LC_3: &[Token] = &[(['k', '\0', '\0', '\0'], 1)];
static LC_4: &[Token] = &[(['c', '\0', '\0', '\0'], 1)];

static LC_ROWS: &[&[Token]] = &[LC_0, LC_1, LC_2, LC_3, LC_4];

const VC_ALLOWED_MASKS: [u16; 8] = [
    (1 << 0) | (1 << 2),
    (1 << 0) | (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2) | (1 << 4),
    0,
    1 << 3,
    1 << 4,
];

#[inline(always)]
fn lookup_mask_optimized(
    rows: &[&[Token]],
    input: &[char],
    input_is_full: bool,
    input_is_complete: bool,
) -> u16 {
    let input_len = input.len() as u8;
    if input_len == 0 {
        return 0;
    }
    let mut ret = 0u16;

    if input_len == 1 {
        let ic0 = input[0];
        for (index, tokens) in rows.iter().enumerate() {
            for (t_chars, t_len) in *tokens {
                if *t_len < 1 || (input_is_full && *t_len > 1) {
                    continue;
                }
                let tc0 = t_chars[0];
                let is_match = ic0 == tc0 || (!input_is_complete && strip_mark(tc0) == ic0);
                if is_match {
                    ret |= 1u16 << index;
                    break;
                }
            }
        }
        return ret;
    }

    if input_len == 2 {
        let ic0 = input[0];
        let ic1 = input[1];
        for (index, tokens) in rows.iter().enumerate() {
            for (t_chars, t_len) in *tokens {
                if *t_len < 2 || (input_is_full && *t_len > 2) {
                    continue;
                }
                let tc0 = t_chars[0];
                let tc1 = t_chars[1];
                let m0 = ic0 == tc0 || (!input_is_complete && strip_mark(tc0) == ic0);
                let m1 = ic1 == tc1 || (!input_is_complete && strip_mark(tc1) == ic1);
                if m0 && m1 {
                    ret |= 1u16 << index;
                    break;
                }
            }
        }
        return ret;
    }

    for (index, tokens) in rows.iter().enumerate() {
        for (t_chars, t_len) in *tokens {
            if *t_len < input_len || (input_is_full && *t_len > input_len) {
                continue;
            }

            let is_match = input
                .iter()
                .zip(t_chars.iter())
                .all(|(&ic, &tc)| ic == tc || (!input_is_complete && strip_mark(tc) == ic));

            if is_match {
                ret |= 1u16 << index;
                break;
            }
        }
    }

    ret
}

#[inline(always)]
const fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
    let mut mask = vo_mask;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        if idx < VC_ALLOWED_MASKS.len() && (VC_ALLOWED_MASKS[idx] & lc_mask) != 0 {
            return true;
        }
        mask &= mask - 1;
    }
    false
}

/// Validates a Vietnamese syllable based on its Consonant-Vowel-Consonant (CVC) structure.
///
/// # Arguments
///
/// * `fc` - First consonant(s).
/// * `vo` - Vowel(s).
/// * `lc` - Last consonant(s).
/// * `full` - Whether the input is considered complete (affects strictness of matching).
pub fn is_valid_cvc(fc: &str, vo: &str, lc: &str, full: bool) -> bool {
    let mut fc_chars = ['\0'; 8];
    let mut vo_chars = ['\0'; 8];
    let mut lc_chars = ['\0'; 8];

    let mut fc_len = 0;
    for c in fc.chars().take(8) {
        fc_chars[fc_len] = c;
        fc_len += 1;
    }
    let mut vo_len = 0;
    for c in vo.chars().take(8) {
        vo_chars[vo_len] = c;
        vo_len += 1;
    }
    let mut lc_len = 0;
    for c in lc.chars().take(8) {
        lc_chars[lc_len] = c;
        lc_len += 1;
    }

    is_valid_cvc_chars(&fc_chars[..fc_len], &vo_chars[..vo_len], &lc_chars[..lc_len], full)
}

/// Low-level variant of [`is_valid_cvc`] that works directly with character slices.
pub fn is_valid_cvc_chars(fc: &[char], vo: &[char], lc: &[char], full: bool) -> bool {
    let fc_mask = if !fc.is_empty() {
        let m = lookup_mask_optimized(FC_ROWS, fc, full || !vo.is_empty(), true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let vo_mask = if !vo.is_empty() {
        let m = lookup_mask_optimized(VO_ROWS, vo, full || !lc.is_empty(), full);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let lc_mask = if !lc.is_empty() {
        let m = lookup_mask_optimized(LC_ROWS, lc, full, true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    if vo_mask == 0 {
        if fc_mask != 0 {
            return true;
        }
        // Consonant abbreviations containing 'đ' (e.g., "đc", "vcđ", "cđ", "bcđ", "nđm", "ađ")
        if fc.iter().any(|&c| matches!(c, 'đ' | 'Đ')) || lc.iter().any(|&c| matches!(c, 'đ' | 'Đ'))
        {
            return true;
        }
        return false;
    }

    if lc_mask != 0 {
        return is_valid_vc(vo_mask, lc_mask);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_syllables() {
        // "ba", "me", "di", "co", "tu"
        assert!(is_valid_cvc("b", "a", "", false));
        assert!(is_valid_cvc("m", "e", "", false));
        assert!(is_valid_cvc("d", "i", "", false));
        assert!(is_valid_cvc("c", "o", "", false));
        assert!(is_valid_cvc("t", "u", "", false));
    }

    #[test]
    fn valid_cvc_with_final_consonant() {
        // "ban", "con", "mat", "toc"
        assert!(is_valid_cvc("b", "a", "n", true));
        assert!(is_valid_cvc("c", "o", "n", true));
        assert!(is_valid_cvc("m", "a", "t", true));
        assert!(is_valid_cvc("t", "o", "c", true));
    }

    #[test]
    fn invalid_consonant_vowel_combinations() {
        // Unknown initial consonant cluster
        assert!(!is_valid_cvc("bb", "a", "", true));
        // Unknown initial consonant cluster
        assert!(!is_valid_cvc("px", "a", "", true));
        // Vowel that doesn't exist in Vietnamese
        assert!(!is_valid_cvc("b", "z", "", true));
    }

    #[test]
    fn valid_no_initial_consonant() {
        // "an", "em", "oi"
        assert!(is_valid_cvc("", "a", "n", true));
        assert!(is_valid_cvc("", "e", "m", true));
        assert!(is_valid_cvc("", "oi", "", false));
    }

    /// Frozen copy of the pre-table `strip_mark` match, kept as the oracle
    /// for the table rewrite. Only the 7 bare marked chars strip; toned
    /// variants (e.g. 'ầ') are identity.
    fn legacy_strip_mark(c: char) -> char {
        match c {
            'ă' | 'â' => 'a',
            'ê' => 'e',
            'ô' | 'ơ' => 'o',
            'ư' => 'u',
            'đ' => 'd',
            _ => c,
        }
    }

    /// Parity: table-backed `strip_mark` must match the legacy match chain,
    /// including the exact-match-only semantics for toned variants.
    #[test]
    fn strip_mark_matches_legacy() {
        use crate::phonetics::VOWELS;
        let mut inputs: Vec<char> = (' '..='~').collect();
        for &c in VOWELS {
            inputs.push(c);
        }
        inputs.extend(['đ', '中', '–']);
        inputs.push('\u{8E2}');
        for c in inputs {
            assert_eq!(strip_mark(c), legacy_strip_mark(c), "strip_mark({c:?})");
        }
    }

    #[test]
    fn valid_chars_variant_matches_str_variant() {
        let cases =
            [("b", "a", "n"), ("t", "oa", ""), ("", "oi", ""), ("kh", "u", ""), ("", "uye", "")];
        for (fc, vo, lc) in cases {
            let str_result = is_valid_cvc(fc, vo, lc, false);
            let fc_chars: Vec<char> = fc.chars().collect();
            let vo_chars: Vec<char> = vo.chars().collect();
            let lc_chars: Vec<char> = lc.chars().collect();
            let chars_result = is_valid_cvc_chars(&fc_chars, &vo_chars, &lc_chars, false);
            assert_eq!(str_result, chars_result, "Mismatch for ({fc:?}, {vo:?}, {lc:?})");
        }
    }
}
