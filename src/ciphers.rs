//! This module contains the implementation of the various ciphers.
#![warn(missing_docs)]

mod cryptarithm;
mod errors;
mod hill;
mod morse;
mod substitution;

use super::cryptogram::Type;
use super::cryptogram::Type::{
    Aristocrat, Caesar, Hill, Identity, Morbit, Patristocrat, PatristocratK1, PatristocratK2, Rot13,
};
pub(crate) use errors::{CipherError, CipherResult, ErrorKind};
use lazy_static::lazy_static;
use rand::prelude::*;

lazy_static! {
    /// Stores words suitable for use as keys in patristocrats or operands in cryptarithms
    static ref WORDS: Vec<String> = {
        let words_file =
            std::env::var("WORDS_FILE").expect("Environment variable WORDS_FILE must be set.");

        log::info!("Loading words from {:?}", words_file);
        let contents = std::fs::read_to_string(words_file).unwrap();

        let words: Vec<_> = contents.trim()
            .split(',')
            .filter_map(|s| {
                // avoid words like "the" or "what" and don't want long, obscure words
                if 4 < s.len() && s.len() < 8 {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert!(!words.is_empty(), "WORDS_FILE must be non-empty");

        words
    };
}

/// Lowercase alphabet.
const ALPHABET: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";

/// Adjust the case of ord to match the case of `to_match`
const fn match_case(ord: u8, to_match: u8) -> u8 {
    let is_lower = (to_match >> 5) & 1;
    ord & !(1 << 5) | (is_lower << 5)
}

/// Convenience function to shift `b` by `by` places.
const fn shift_letter(b: u8, by: u8) -> u8 {
    let offset = match_case(b'a', b);

    (b - offset + by) % 26 + offset
}

/// Returns the input string unchanged.
fn identity(s: &str) -> Cipher {
    Cipher::new(s.to_string(), None)
}

/// The type returned by the various encryption functions in this library.
#[derive(Debug)]
pub struct Cipher {
    /// The ciphertext
    pub ciphertext: String,
    /// The key
    pub key: Option<String>,
}

impl Cipher {
    fn new(ciphertext: String, key: Option<String>) -> Self {
        Self { ciphertext, key }
    }

    /// Wrapper function to call a specific cipher by [`Type`].
    pub(crate) fn encrypt(
        plaintext: &str,
        cipher_type: Type,
        key: Option<String>,
    ) -> CipherResult<Self> {
        let rng = &mut thread_rng();

        Ok(match cipher_type {
            Aristocrat => substitution::aristocrat(plaintext, rng),
            Caesar => substitution::caeser(plaintext, rng),
            // Cryptarithm => cryptarithm::cryptarithm(&mut rng),
            Hill => hill::hill(plaintext, key, rng)?,
            Identity => identity(plaintext),
            Morbit => morse::morbit(plaintext, key),
            Patristocrat => substitution::patristocrat(plaintext, rng),
            PatristocratK1 => substitution::patristocrat_k1(plaintext, key, rng),
            PatristocratK2 => substitution::patristocrat_k2(plaintext, key, rng),
            Rot13 => substitution::rot13(plaintext),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_case_same_case() {
        assert_eq!(match_case(b'a', b'a'), b'a');
    }

    #[test]
    fn test_match_case_lower() {
        let inps: Vec<_> = (b'a'..=b'z').collect();

        for inp in inps {
            assert_eq!(
                match_case(inp, inp.to_ascii_uppercase()),
                inp.to_ascii_uppercase()
            );
        }
    }

    #[test]
    fn test_match_case_upper() {
        let inps: Vec<_> = (b'A'..=b'Z').collect();

        for inp in inps {
            assert_eq!(
                match_case(inp, inp.to_ascii_lowercase()),
                inp.to_ascii_lowercase()
            );
        }
    }

    #[test]
    fn test_shift_letter() {
        // build a rotated alphabet
        let mut expected = Vec::new();
        for i in b't'..=b'z' {
            expected.push(i);
        }
        for i in b'a'..b't' {
            expected.push(i);
        }

        println!("{expected:?}");
        let initial = b't';
        for i in 0..26 {
            assert_eq!(shift_letter(initial, i), expected[i as usize]);
        }
    }

    #[test]
    fn test_identity() {
        assert_eq!(
            identity("abcdefghijklmnopqrstuvwxyz").ciphertext,
            "abcdefghijklmnopqrstuvwxyz".to_string()
        )
    }
}
