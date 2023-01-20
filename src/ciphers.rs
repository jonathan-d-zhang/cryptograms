//! This module contains the implementation of the ciphers.

#![warn(missing_docs)]

pub use super::cryptogram::Type;
use super::cryptogram::Type::*;
use rand::prelude::*;
use lazy_static::lazy_static;

mod cryptarithm;
mod morse;
mod substitution;

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
                if 3 < s.len() && s.len() < 8 {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        if words.is_empty() {
            panic!("WORDS_FILE must be non-empty");
        }

        words
    };
}

/// Lowercase alphabet.
const ALPHABET: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";


/// Adjust the case of ord to match the case of to_match
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
fn identity(s: &str) -> String {
    s.to_string()
}

/// Wrapper function to call a specific cipher by [`Type`].
pub fn encrypt(plaintext: &str, cipher_type: Type, key: Option<String>) -> String {
    let rng = &mut thread_rng();
    use substitution::*;
    match cipher_type {
        Identity => identity(plaintext),
        Rot13 => rot13(plaintext),
        Caesar => caeser(plaintext, rng),
        Aristocrat => aristocrat(plaintext, rng),
        Patristocrat => patristocrat(plaintext, rng),
        K1Patristocrat => patristocrat_k1(plaintext, key, rng),
        Morbit => morse::morbit(plaintext, key),
        //Cryptarithm => cryptarithm::cryptarithm(&mut rng),
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

        println!("{:?}", expected);
        let initial = b't';
        for i in 0..26 {
            assert_eq!(shift_letter(initial, i), expected[i as usize]);
        }
    }
}
