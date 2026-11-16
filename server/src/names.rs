//! Display names from building blocks (ADR 0029).
//!
//! A name is not free text: it is an adjective and a noun from fixed
//! lists, with an optional number, stored as their ids
//! (`a<adjective>.n<noun>` or `a<adjective>.n<noun>.<number>`, e.g.
//! `a3.n12` or `a3.n12.7`). The words live only in the web app's texts
//! (`web/src/i18n/de.ts` and `en.ts`, `names`), so every viewer reads a
//! name in their own language. The server only checks the form and that
//! the ids are within the lists; it never stores text a player typed.

use crate::error::{ApiError, ApiResult};

/// Number of adjectives (`names.adjectives` in the web texts). The lists
/// only grow at the end, so stored ids keep their meaning.
pub const ADJECTIVES: u32 = 28;
/// Number of nouns (`names.nouns` in the web texts).
pub const NOUNS: u32 = 28;
/// Largest number after the name (1 to this; none is fine too).
pub const MAX_NUMBER: u32 = 99;

/// A decimal index without sign or leading zero, below `limit`.
fn index(digits: &str, limit: u32) -> Option<u32> {
    let canonical = digits == "0"
        || (!digits.is_empty()
            && digits.len() <= 3
            && digits.bytes().all(|b| b.is_ascii_digit())
            && !digits.starts_with('0'));
    if !canonical {
        return None;
    }
    digits.parse::<u32>().ok().filter(|&n| n < limit)
}

/// Whether a stored or sent name is a valid id (`a3.n12`, `a3.n12.7`).
pub fn is_valid(name: &str) -> bool {
    let mut parts = name.split('.');
    let adjective = parts.next().and_then(|p| p.strip_prefix('a'));
    let noun = parts.next().and_then(|p| p.strip_prefix('n'));
    let number = parts.next();
    if parts.next().is_some() {
        return false;
    }
    let blocks = adjective.and_then(|a| index(a, ADJECTIVES)).is_some()
        && noun.and_then(|n| index(n, NOUNS)).is_some();
    let number_ok = number.is_none_or(|n| index(n, MAX_NUMBER + 1).is_some_and(|n| n >= 1));
    blocks && number_ok
}

/// The name a request sent, checked: `Ok(None)` removes it (empty or only
/// spaces), `Ok(Some(id))` sets it; anything else is a 400 `name`.
pub fn parse(raw: &str) -> ApiResult<Option<String>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    if is_valid(raw) {
        Ok(Some(raw.to_owned()))
    } else {
        Err(ApiError::bad_request("name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_within_the_lists_are_names() {
        assert!(is_valid("a0.n0"));
        assert!(is_valid("a3.n12"));
        assert!(is_valid("a27.n27"));
        assert!(is_valid("a27.n27.99"));
        assert!(is_valid("a5.n9.1"));
    }

    #[test]
    fn everything_else_is_refused() {
        for bad in [
            "",
            "Jan",
            "a28.n0",
            "a0.n28",
            "a0.n0.0",
            "a0.n0.100",
            "a01.n2",
            "a1.n02",
            "a1.n2.07",
            "a1.n2.",
            "a1.n2.3.4",
            "a-1.n2",
            "a+1.n2",
            "n1.a2",
            "a1",
            "a1.n",
            "a.n1",
            " a1.n2",
            "a1.n2 ",
            "A1.N2",
            "a1.n2x",
            "a１.n2",
            "a9999999999.n1",
        ] {
            assert!(!is_valid(bad), "{bad:?} passed");
        }
    }

    #[test]
    fn requests_are_trimmed_and_empty_removes() {
        assert_eq!(parse("").ok(), Some(None));
        assert_eq!(parse("   ").ok(), Some(None));
        assert_eq!(parse(" a1.n2 ").ok(), Some(Some("a1.n2".into())));
        assert_eq!(parse("a1.n2.42").ok(), Some(Some("a1.n2.42".into())));
        assert!(parse("Der alte Fritz").is_err());
    }
}
