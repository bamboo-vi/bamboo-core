//! Vietnamese spelling validation logic based on CVC (Consonant-Vowel-Consonant) structure.

// Static token representation: (chars, length)
type Token = ([char; 4], u8);

/// A spelling token with its characters pre-packed into `u16` lanes.
/// `raw[k]` / `stripped[k]` hold lane `k` (0 for unused lanes); `len` is the
/// token length (always 1..=3, verified by `packed_token_tables_wellformed`).
/// All token chars are BMP (max U+1EF9), so `as u16` is exact — also verified
/// there. Lets the matcher test whole lanes with integer compares instead of
/// per-character branches plus `strip_mark` calls.
#[derive(Clone, Copy)]
struct PackedToken {
    raw: [u16; 3],
    stripped: [u16; 3],
    len: u8,
}

impl PackedToken {
    const fn pack(chars: [char; 4], len: u8) -> Self {
        let mut raw = [0u16; 3];
        let mut stripped = [0u16; 3];
        let mut k = 0usize;
        while k < 3 {
            let c = chars[k];
            raw[k] = c as u16;
            stripped[k] = strip_mark(c) as u16;
            k += 1;
        }
        Self { raw, stripped, len }
    }
}

/// Builds a packed row from a char token row at compile time, keeping the
/// char lists the single source of truth.
const fn pack_row<const N: usize>(src: &[Token; N]) -> [PackedToken; N] {
    let mut out = [PackedToken { raw: [0; 3], stripped: [0; 3], len: 0 }; N];
    let mut i = 0usize;
    while i < N {
        let (chars, len) = src[i];
        out[i] = PackedToken::pack(chars, len);
        i += 1;
    }
    out
}

/// Single token-lane test with no branches: raw hit, or stripped hit while
/// the input is still incomplete (mid-typing bases like 'u' match 'ư').
#[inline(always)]
const fn lane_eq(input: u16, raw: u16, stripped: u16, incomplete: bool) -> bool {
    (input == raw) | (incomplete & (input == stripped))
}

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
static FC: [Token; 29] = [
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

/// Char-row aggregates, kept as the readable source of truth and the legacy
/// oracle input. Production matches on the packed parallels below.
#[cfg(test)]
static FC_ROWS: &[&[Token]] = &[&FC];

static VO_0: [Token; 6] = [
    (['ê', '\0', '\0', '\0'], 1),
    (['i', '\0', '\0', '\0'], 1),
    (['u', 'a', '\0', '\0'], 2),
    (['u', 'ê', '\0', '\0'], 2),
    (['u', 'y', '\0', '\0'], 2),
    (['y', '\0', '\0', '\0'], 1),
];
static VO_1: [Token; 5] = [
    (['a', '\0', '\0', '\0'], 1),
    (['i', 'ê', '\0', '\0'], 2),
    (['o', 'a', '\0', '\0'], 2),
    (['u', 'y', 'ê', '\0'], 3),
    (['y', 'ê', '\0', '\0'], 2),
];
static VO_2: [Token; 13] = [
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
static VO_3: [Token; 1] = [(['o', 'ă', '\0', '\0'], 2)];
static VO_4: [Token; 1] = [(['u', 'ơ', '\0', '\0'], 2)];
static VO_5: [Token; 30] = [
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
static VO_6: [Token; 2] = [(['ă', '\0', '\0', '\0'], 1), (['u', '\0', '\0', '\0'], 1)];
static VO_7: [Token; 1] = [(['i', '\0', '\0', '\0'], 1)];

#[cfg(test)]
static VO_ROWS: &[&[Token]] = &[&VO_0, &VO_1, &VO_2, &VO_3, &VO_4, &VO_5, &VO_6, &VO_7];

static LC_0: [Token; 2] = [(['c', 'h', '\0', '\0'], 2), (['n', 'h', '\0', '\0'], 2)];
static LC_1: [Token; 2] = [(['c', '\0', '\0', '\0'], 1), (['n', 'g', '\0', '\0'], 2)];
static LC_2: [Token; 4] = [
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['p', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
];
static LC_3: [Token; 1] = [(['k', '\0', '\0', '\0'], 1)];
static LC_4: [Token; 1] = [(['c', '\0', '\0', '\0'], 1)];

#[cfg(test)]
static LC_ROWS: &[&[Token]] = &[&LC_0, &LC_1, &LC_2, &LC_3, &LC_4];

// Packed parallels, derived from the char rows above at compile time.
static FC_PACKED: [PackedToken; 29] = pack_row(&FC);
static VO_0_PACKED: [PackedToken; 6] = pack_row(&VO_0);
static VO_1_PACKED: [PackedToken; 5] = pack_row(&VO_1);
static VO_2_PACKED: [PackedToken; 13] = pack_row(&VO_2);
static VO_3_PACKED: [PackedToken; 1] = pack_row(&VO_3);
static VO_4_PACKED: [PackedToken; 1] = pack_row(&VO_4);
static VO_5_PACKED: [PackedToken; 30] = pack_row(&VO_5);
static VO_6_PACKED: [PackedToken; 2] = pack_row(&VO_6);
static VO_7_PACKED: [PackedToken; 1] = pack_row(&VO_7);
static LC_0_PACKED: [PackedToken; 2] = pack_row(&LC_0);
static LC_1_PACKED: [PackedToken; 2] = pack_row(&LC_1);
static LC_2_PACKED: [PackedToken; 4] = pack_row(&LC_2);
static LC_3_PACKED: [PackedToken; 1] = pack_row(&LC_3);
static LC_4_PACKED: [PackedToken; 1] = pack_row(&LC_4);
static FC_PACKED_ROWS: &[&[PackedToken]] = &[&FC_PACKED];
static VO_PACKED_ROWS: &[&[PackedToken]] = &[
    &VO_0_PACKED,
    &VO_1_PACKED,
    &VO_2_PACKED,
    &VO_3_PACKED,
    &VO_4_PACKED,
    &VO_5_PACKED,
    &VO_6_PACKED,
    &VO_7_PACKED,
];
static LC_PACKED_ROWS: &[&[PackedToken]] =
    &[&LC_0_PACKED, &LC_1_PACKED, &LC_2_PACKED, &LC_3_PACKED, &LC_4_PACKED];

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
    rows: &[&[PackedToken]],
    input: &[char],
    input_is_full: bool,
    input_is_complete: bool,
) -> u16 {
    let input_len = input.len();
    // No token exceeds 3 chars, so longer inputs match nothing. Astral input
    // chars exceed the u16 lanes yet can never equal a BMP token lane, so an
    // input containing one matches nothing either (both verified by parity).
    if input_len == 0 || input_len > 3 || input.iter().any(|&c| (c as u32) > 0xFFFF) {
        return 0;
    }
    let incomplete = !input_is_complete;
    let mut packed = [0u16; 3];
    for (k, &c) in input.iter().enumerate() {
        packed[k] = c as u16;
    }
    let need = input_len as u8;
    let mut ret = 0u16;
    for (index, tokens) in rows.iter().enumerate() {
        for t in *tokens {
            if t.len < need || (input_is_full && t.len != need) {
                continue;
            }
            // Branchless lanes; a single branch per token keeps the early
            // exit that pays off on the dominant valid-syllable path.
            let mut hit = true;
            for (k, &in_lane) in packed[..input_len].iter().enumerate() {
                hit &= lane_eq(in_lane, t.raw[k], t.stripped[k], incomplete);
            }
            if hit {
                ret |= 1u16 << index;
                break;
            }
        }
    }
    ret
}

const fn build_vc_ok() -> [u16; 256] {
    let mut t = [0u16; 256];
    let mut m = 0usize;
    while m < 256 {
        let mut acc = 0u16;
        let mut bits = m;
        let mut b = 0usize;
        while bits != 0 {
            if bits & 1 != 0 && b < VC_ALLOWED_MASKS.len() {
                acc |= VC_ALLOWED_MASKS[b];
            }
            bits >>= 1;
            b += 1;
        }
        t[m] = acc;
        m += 1;
    }
    t
}

/// `VC_OK[mask]` = OR of [`VC_ALLOWED_MASKS`] over the mask's set bits.
/// Replaces the trailing-zeros loop in [`is_valid_vc`] with one load.
/// Bits at index 8+ contribute nothing, exactly like the loop's guard.
const VC_OK: [u16; 256] = build_vc_ok();

#[inline(always)]
const fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
    (VC_OK[(vo_mask & 0xFF) as usize] & lc_mask) != 0
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
        let m = lookup_mask_optimized(FC_PACKED_ROWS, fc, full || !vo.is_empty(), true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let vo_mask = if !vo.is_empty() {
        let m = lookup_mask_optimized(VO_PACKED_ROWS, vo, full || !lc.is_empty(), full);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let lc_mask = if !lc.is_empty() {
        let m = lookup_mask_optimized(LC_PACKED_ROWS, lc, full, true);
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

    /// Frozen copies of the pre-packed implementations, kept as the oracle
    /// for the packed-token rewrite. Do NOT rewire these to tables/packing.
    mod legacy {
        use super::{Token, VC_ALLOWED_MASKS};

        pub fn strip_mark(c: char) -> char {
            match c {
                'ă' | 'â' => 'a',
                'ê' => 'e',
                'ô' | 'ơ' => 'o',
                'ư' => 'u',
                'đ' => 'd',
                _ => c,
            }
        }

        pub fn lookup_mask(
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
            for (index, tokens) in rows.iter().enumerate() {
                for (t_chars, t_len) in *tokens {
                    if *t_len < input_len || (input_is_full && *t_len > input_len) {
                        continue;
                    }
                    let hit = input
                        .iter()
                        .zip(t_chars.iter())
                        .all(|(&ic, &tc)| ic == tc || (!input_is_complete && strip_mark(tc) == ic));
                    if hit {
                        ret |= 1u16 << index;
                        break;
                    }
                }
            }
            ret
        }

        pub const fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
            let mut mask = vo_mask;
            let mut ok = false;
            while mask != 0 {
                let idx = mask.trailing_zeros() as usize;
                if idx < VC_ALLOWED_MASKS.len() && (VC_ALLOWED_MASKS[idx] & lc_mask) != 0 {
                    ok = true;
                }
                mask &= mask - 1;
            }
            ok
        }
    }

    /// Frozen copy of the pre-table `strip_mark` match, kept as the oracle
    /// for the table rewrite. Only the 7 bare marked chars strip; toned
    /// variants (e.g. 'ầ') are identity.
    fn legacy_strip_mark(c: char) -> char {
        legacy::strip_mark(c)
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

    /// Parity: packed lookup must answer exactly like the legacy per-char
    /// matcher on every input shape, including mixed raw/stripped lanes,
    /// astral chars (never match BMP tokens), and over-long inputs.
    ///
    /// Characterization harness for a behavior-preserving rewrite: passes on
    /// the current code (validating the harness) and must stay green after
    /// the packing.
    #[test]
    fn packed_matches_legacy_lookup() {
        const SET: [char; 12] = ['a', 'n', 'g', 'â', 'ê', 'ô', 'ơ', 'ư', 'đ', 'o', 'u', 'e'];
        let packed_groups: &[&[&[PackedToken]]] = &[FC_PACKED_ROWS, VO_PACKED_ROWS, LC_PACKED_ROWS];
        let groups: &[&[&[Token]]] = &[FC_ROWS, VO_ROWS, LC_ROWS];
        // Length 1..=3 exhaustive over SET x {full, complete} x row groups.
        for len in 1..=3usize {
            let mut idx = vec![0usize; len];
            loop {
                let input: Vec<char> = idx.iter().map(|&k| SET[k]).collect();
                for full in [false, true] {
                    for complete in [false, true] {
                        for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                            assert_eq!(
                                lookup_mask_optimized(packed_rows, &input, full, complete),
                                legacy::lookup_mask(rows, &input, full, complete),
                                "lookup {input:?} full={full} complete={complete}"
                            );
                        }
                    }
                }
                let mut pos = 0;
                loop {
                    idx[pos] += 1;
                    if idx[pos] < SET.len() {
                        break;
                    }
                    idx[pos] = 0;
                    pos += 1;
                    if pos >= len {
                        break;
                    }
                }
                if pos >= len {
                    break;
                }
            }
        }

        // Explicit mixed-lane case: token ('u','â','y') matches ['u','a','y']
        // only via the stripped middle lane when incomplete.
        let mixed = ['u', 'a', 'y'];
        let hit = lookup_mask_optimized(VO_PACKED_ROWS, &mixed, false, false);
        assert_eq!(hit, legacy::lookup_mask(VO_ROWS, &mixed, false, false));
        assert_ne!(hit & (1u16 << 5), 0, "VO_5 mixed row must hit");
        let hit_complete = lookup_mask_optimized(VO_PACKED_ROWS, &mixed, false, true);
        assert_eq!(hit_complete, legacy::lookup_mask(VO_ROWS, &mixed, false, true));
        assert_eq!(hit_complete & (1u16 << 5), 0, "VO_5 must miss when complete");

        // Astral chars never match BMP tokens (both sides must agree on 0
        // for any length; guards future packing truncation).
        for astral in ['\u{10061}', '\u{10437}', '\u{1F600}'] {
            for input in [&[astral][..], &[astral, 'i'][..], &['a', astral][..]] {
                for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                    assert_eq!(
                        lookup_mask_optimized(packed_rows, input, false, false),
                        0,
                        "astral {input:?} must match nothing"
                    );
                    assert_eq!(
                        legacy::lookup_mask(rows, input, false, false),
                        0,
                        "legacy astral {input:?} must match nothing"
                    );
                }
            }
        }

        // Over-long inputs match nothing (all tokens are shorter).
        for input in [&['a', 'b', 'c', 'd'][..], &['ư', 'ơ', 'i', 'a', 'b'][..]] {
            for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                assert_eq!(lookup_mask_optimized(packed_rows, input, false, false), 0);
                assert_eq!(legacy::lookup_mask(rows, input, false, false), 0);
            }
        }

        // VC table parity over the full u8 vo-mask space x lc samples.
        for vo in 0..=255u16 {
            for lc in [0u16, 1, 3, 7, 15, 31, 63, 255, 0xFFFF] {
                assert_eq!(is_valid_vc(vo, lc), legacy::is_valid_vc(vo, lc), "vc vo={vo} lc={lc}");
            }
        }
    }

    /// Guards the packing preconditions for future token edits: every token
    /// is 1..=3 chars, in-lane chars are non-NUL BMP, padding lanes are NUL.
    #[test]
    fn packed_token_tables_wellformed() {
        let groups: &[&[&[Token]]] = &[FC_ROWS, VO_ROWS, LC_ROWS];
        for rows in groups {
            for tokens in *rows {
                for (chars, len) in *tokens {
                    assert!((1..=3u8).contains(len), "token len {len} out of range");
                    for (k, &c) in chars.iter().enumerate() {
                        if (k as u8) < *len {
                            assert_ne!(c, '\0', "in-lane NUL");
                            assert!((c as u32) <= 0xFFFF, "non-BMP token char {c:?}");
                        } else {
                            assert_eq!(c, '\0', "padding lane must be NUL");
                        }
                    }
                }
            }
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
