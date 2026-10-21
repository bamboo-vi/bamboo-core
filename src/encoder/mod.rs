//! Provides functions and tables for encoding Vietnamese Unicode text into legacy character sets.
//!
//! Supported character sets include:
//! - **Unicode** (precomposed NFC)
//! - **Unicode tổ hợp** (decomposed NFD)
//! - **TCVN3 (ABC)**
//! - **VNI Windows**
//! - **Windows 1258 codepage**
//! - **VIQR**
//! - **VISCII**
//! - **VPS**
//! - **BKHCM 1** / **BKHCM 2**
//! - **Vietware X** / **Vietware Full**
//! - **UTF-8**
//! - **NCR Decimal** / **NCR Hex**
//! - **Unicode C string Hex** / **Unicode C string Decimal**

pub mod tables;
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
pub use tables::{get_charset_definition, get_charset_definitions};

static UNICODE: &str = "Unicode";

/// Strongly-typed Vietnamese character sets supported by Bamboo Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Charset {
    /// Canonical Unicode precomposed NFC (default).
    #[default]
    Unicode,
    /// TCVN3 (ABC standard).
    Tcvn3,
    /// VNI Windows legacy encoding.
    VniWindows,
    /// Unicode decomposed NFD (Unicode tổ hợp).
    UnicodeCompound,
    /// Windows 1258 Vietnamese codepage.
    Windows1258,
    /// VIQR (Vietnamese Quoted-Readable).
    Viqr,
    /// VISCII (Vietnamese Standard Code for Information Interchange).
    Viscii,
    /// VPS legacy encoding.
    Vps,
    /// BKHCM 1 legacy encoding.
    Bkhcm1,
    /// BKHCM 2 legacy encoding.
    Bkhcm2,
    /// Vietware X encoding.
    VietwareX,
    /// Vietware Full encoding.
    VietwareFull,
    /// UTF-8 literal byte escape sequences.
    Utf8,
    /// NCR Decimal (`&#...;`).
    NcrDecimal,
    /// NCR Hex (`&#x...;`).
    NcrHex,
    /// Unicode C string Hex (`\x...`).
    UnicodeCStringHex,
    /// Unicode C string Decimal (`\u...`).
    UnicodeCStringDecimal,
}

impl Charset {
    /// Returns the canonical display name of the charset definition.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unicode => "Unicode",
            Self::Tcvn3 => "TCVN3 (ABC)",
            Self::VniWindows => "VNI Windows",
            Self::UnicodeCompound => "Unicode tổ hợp",
            Self::Windows1258 => "Windows 1258 codepage",
            Self::Viqr => "VIQR",
            Self::Viscii => "VISCII",
            Self::Vps => "VPS",
            Self::Bkhcm1 => "BKHCM 1",
            Self::Bkhcm2 => "BKHCM 2",
            Self::VietwareX => "Vietware X",
            Self::VietwareFull => "Vietware Full",
            Self::Utf8 => "UTF-8",
            Self::NcrDecimal => "NCR Decimal",
            Self::NcrHex => "NCR Hex",
            Self::UnicodeCStringHex => "Unicode C string Hex",
            Self::UnicodeCStringDecimal => "Unicode C string Decimal",
        }
    }
}

impl fmt::Display for Charset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Charset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unicode" | "unicode" => Ok(Self::Unicode),
            "TCVN3 (ABC)" | "TCVN3" | "tcvn3" => Ok(Self::Tcvn3),
            "VNI Windows" | "VNI" | "vni" => Ok(Self::VniWindows),
            "Unicode tổ hợp" | "UnicodeToHop" => Ok(Self::UnicodeCompound),
            "Windows 1258 codepage" | "Windows-1258" | "windows-1258" => Ok(Self::Windows1258),
            "VIQR" | "viqr" => Ok(Self::Viqr),
            "VISCII" | "viscii" => Ok(Self::Viscii),
            "VPS" | "vps" => Ok(Self::Vps),
            "BKHCM 1" | "bkhcm1" => Ok(Self::Bkhcm1),
            "BKHCM 2" | "bkhcm2" => Ok(Self::Bkhcm2),
            "Vietware X" | "vietware_x" => Ok(Self::VietwareX),
            "Vietware Full" | "vietware_full" => Ok(Self::VietwareFull),
            "UTF-8" | "utf-8" | "utf8" => Ok(Self::Utf8),
            "NCR Decimal" | "ncr_decimal" => Ok(Self::NcrDecimal),
            "NCR Hex" | "ncr_hex" => Ok(Self::NcrHex),
            "Unicode C string Hex" => Ok(Self::UnicodeCStringHex),
            "Unicode C string Decimal" => Ok(Self::UnicodeCStringDecimal),
            _ => Err(()),
        }
    }
}

/// Encodes a Vietnamese Unicode string into a strongly-typed [`Charset`].
///
/// Returns [`Cow::Borrowed`] with **zero heap allocations** if `charset` is [`Charset::Unicode`],
/// if `input` is empty, or if `input` contains no Vietnamese characters requiring encoding.
///
/// # Example
/// ```rust
/// use bamboo_core::{encode_charset, Charset};
///
/// let encoded = encode_charset(Charset::Viqr, "tiếng Việt");
/// assert_eq!(encoded, "tie^'ng Vie^.t");
///
/// // Zero-allocation on plain ASCII or standard Unicode
/// let passthrough = encode_charset(Charset::Viqr, "hello world");
/// assert!(matches!(passthrough, std::borrow::Cow::Borrowed(_)));
/// ```
pub fn encode_charset<'a>(charset: Charset, input: &'a str) -> Cow<'a, str> {
    if charset == Charset::Unicode || input.is_empty() {
        return Cow::Borrowed(input);
    }

    match get_charset_definition(charset.as_str()) {
        Some(charset_def) => {
            if !input.chars().any(|c| !c.is_ascii() && charset_def.contains_key(&c)) {
                return Cow::Borrowed(input);
            }

            let mut output = String::with_capacity(input.len());
            for char in input.chars() {
                if char.is_ascii() {
                    output.push(char);
                } else {
                    match charset_def.get(&char) {
                        Some(encoded) => output.push_str(encoded),
                        None => output.push(char),
                    }
                }
            }
            Cow::Owned(output)
        }
        None => Cow::Borrowed(input),
    }
}

/// Encodes a Vietnamese Unicode string into a specific character set by string name.
///
/// If `charset_name` is `"Unicode"` or the input is empty, returns a clone of `input`.
/// Unrecognized characters or unknown charset names pass through unchanged.
///
/// For zero allocations, prefer [`encode_charset`] with the typed [`Charset`] enum.
///
/// # Arguments
/// * `charset_name` - Name of the target character set (e.g., `"TCVN3 (ABC)"`, `"VNI Windows"`, `"VIQR"`).
/// * `input` - The source Unicode string.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::encode;
///
/// let viqr = encode("VIQR", "tiếng Việt");
/// assert_eq!(viqr, "tie^'ng Vie^.t");
/// ```
pub fn encode(charset_name: &str, input: &str) -> String {
    if charset_name == UNICODE || input.is_empty() {
        return input.to_string();
    }

    match get_charset_definition(charset_name) {
        Some(charset_def) => {
            let mut output = String::with_capacity(input.len());
            for char in input.chars() {
                if char.is_ascii() {
                    output.push(char);
                } else {
                    match charset_def.get(&char) {
                        Some(encoded) => output.push_str(encoded),
                        None => output.push(char),
                    }
                }
            }
            output
        }
        None => input.to_string(),
    }
}

/// Returns an iterator over all supported character set names without heap allocations.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::charset_names;
///
/// let names: Vec<&'static str> = charset_names().collect();
/// assert!(names.contains(&"Unicode"));
/// assert!(names.contains(&"TCVN3 (ABC)"));
/// assert!(names.contains(&"VIQR"));
/// ```
pub fn charset_names() -> impl Iterator<Item = &'static str> {
    std::iter::once(UNICODE).chain(get_charset_definitions().keys().copied())
}

/// Returns a newly allocated list of all supported character set names.
pub fn get_charset_name() -> Vec<String> {
    charset_names().map(String::from).collect()
}

/// Alias for [`get_charset_name`].
pub fn get_charset_names() -> Vec<String> {
    get_charset_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_charset_zero_alloc() {
        // Plain ASCII returns Borrowed (0 heap allocation)
        let res = encode_charset(Charset::Viqr, "hello world");
        assert_eq!(res, "hello world");
        assert!(matches!(res, Cow::Borrowed(_)));

        // Unicode charset returns Borrowed (0 heap allocation)
        let res_uni = encode_charset(Charset::Unicode, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert_eq!(res_uni, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert!(matches!(res_uni, Cow::Borrowed(_)));

        // Converted charset returns Owned with correct transformed string
        let res_viqr = encode_charset(Charset::Viqr, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert_eq!(res_viqr, "tie^'ng Vie^.t");
        assert!(matches!(res_viqr, Cow::Owned(_)));
    }

    #[test]
    fn test_windows_1258_matches_the_codepage() {
        // Bytes DD EC and D0, as glibc iconv writes them, shown as cp1252 like the rest of the table.
        assert_eq!(encode_charset(Charset::Windows1258, "Ứ"), "Ýì");
        assert_eq!(encode_charset(Charset::Windows1258, "Đ"), "\u{D0}");

        // Every code must be bytes of the codepage, i.e. characters cp1252 can show.
        let cp1252_high = "€‚ƒ„…†‡ˆ‰Š‹ŒŽ‘’“”•–—˜™š›œžŸ";
        let def = get_charset_definition(Charset::Windows1258.as_str()).unwrap();
        for (letter, code) in def.entries() {
            for c in code.chars() {
                assert!(c <= '\u{FF}' || cp1252_high.contains(c), "{letter} => {code:?}");
            }
        }
    }
}
