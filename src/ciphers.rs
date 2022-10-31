//! This module contains the implementation of the ciphers.

#![warn(missing_docs)]

use super::cryptogram::Type;
use super::cryptogram::Type::*;
use rand::prelude::*;

mod morse;
mod substitution;
mod cryptarithm;

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
    let mut rng = thread_rng();
    match cipher_type {
        Identity => identity(plaintext),
        Rot13 => substitution::rot13(plaintext),
        Caesar => substitution::caeser(plaintext, &mut rng),
        Aristocrat => substitution::aristocrat(plaintext, &mut rng),
        Morbit => morse::morbit(plaintext, key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mock_rng;
    pub use mock_rng::MockRng;

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
