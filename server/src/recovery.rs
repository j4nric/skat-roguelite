//! The recovery code: shown once at sign-up, the way back in without a
//! passkey (lost phone, new domain). 24 characters of Crockford base32
//! (120 bits), shown in six groups of four: `ABCD-EFGH-…`.
//!
//! The first 8 characters (40 bits) are a public lookup id stored as is,
//! so the account can be found; the other 16 characters (80 bits) are the
//! secret, stored only as an argon2id hash. Using the code replaces it by
//! a new one.

use argon2::Argon2;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/// Crockford base32: no I, L, O, U.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Characters in a code.
pub const CODE_LEN: usize = 24;
/// Characters of the public lookup id.
const ID_LEN: usize = 8;

/// A fresh recovery code: (code as shown, lookup id, secret).
pub struct NewCode {
    /// Formatted for the player (`ABCD-EFGH-JKMN-PQRS-TVWX-YZ01`).
    pub display: String,
    /// Public lookup id (first 8 characters).
    pub id: String,
    /// The secret part (last 16 characters), to be hashed.
    pub secret: String,
}

/// Make a new random code.
pub fn generate() -> NewCode {
    let mut bytes = [0u8; CODE_LEN];
    OsRng.fill_bytes(&mut bytes);
    // 256 is a multiple of 32, so `b % 32` is uniform.
    let raw: String = bytes
        .iter()
        .map(|b| char::from(ALPHABET[usize::from(b % 32)]))
        .collect();
    let display = raw
        .as_bytes()
        .chunks(4)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("-");
    NewCode {
        display,
        id: raw[..ID_LEN].to_owned(),
        secret: raw[ID_LEN..].to_owned(),
    }
}

/// Read a typed code: case, spaces and hyphens do not matter, and the
/// letters Crockford maps (O to 0, I and L to 1) are forgiven. Returns
/// (id, secret) or None if it cannot be a code.
pub fn parse(input: &str) -> Option<(String, String)> {
    let mut raw = String::with_capacity(CODE_LEN);
    for c in input.chars() {
        let c = c.to_ascii_uppercase();
        let c = match c {
            'O' => '0',
            'I' | 'L' => '1',
            c => c,
        };
        if c.is_ascii() && ALPHABET.contains(&(c as u8)) {
            raw.push(c);
        } else if c == '-' || c.is_whitespace() {
            continue;
        } else {
            return None;
        }
        if raw.len() > CODE_LEN {
            return None;
        }
    }
    if raw.len() != CODE_LEN {
        return None;
    }
    Some((raw[..ID_LEN].to_owned(), raw[ID_LEN..].to_owned()))
}

/// Hash the secret (argon2id, default parameters, random salt).
pub fn hash(secret: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .ok()
        .map(|h| h.to_string())
}

/// Whether the secret matches the stored hash.
pub fn verify(secret: &str, stored: &str) -> bool {
    PasswordHash::new(stored)
        .map(|h| {
            Argon2::default()
                .verify_password(secret.as_bytes(), &h)
                .is_ok()
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_parses_back() {
        let code = generate();
        assert_eq!(code.display.len(), CODE_LEN + 5);
        let (id, secret) = parse(&code.display).expect("parses");
        assert_eq!(id, code.id);
        assert_eq!(secret, code.secret);
        // Lower case and spaces are fine.
        let typed = code.display.to_lowercase().replace('-', " ");
        assert_eq!(parse(&typed), Some((code.id, code.secret)));
    }

    #[test]
    fn wrong_codes_are_rejected() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("ABCD"), None);
        assert_eq!(parse("ABCD-EFGH-JKMN-PQRS-TVWX-YZ01-2"), None);
        assert_eq!(parse("ABCD-EFGH-JKMN-PQRS-TVWX-YZ0!"), None);
    }

    #[test]
    fn hash_verifies_only_the_secret() {
        let h = hash("0123456789ABCDEF").expect("hashes");
        assert!(verify("0123456789ABCDEF", &h));
        assert!(!verify("0123456789ABCDEG", &h));
        assert!(!verify("0123456789ABCDEF", "not a hash"));
    }
}
