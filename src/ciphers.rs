//! This module contains the implementation of the ciphers.

#![warn(missing_docs)]

use rand::prelude::*;
use super::cryptogram::Type;
use super::cryptogram::Type::*;

/// Lowercase alphabet.
const ALPHABET: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";


/// Adjust the case of ord to match the case of to_match
fn match_case(ord: u8, to_match: u8) -> u8 {
    let is_lower = (to_match >> 5) & 1;
    ord & !(1 << 5) | (is_lower << 5)
}

/// Wrapper function to call a specific cipher by [`Type`].
pub fn encrypt(plaintext: &str, cipher_type: Type) -> String {
    match cipher_type {
        Identity => identity(plaintext),
        Rot13 => rot13(plaintext),
        Aristocrat => aristocrat(plaintext, &mut thread_rng()),
    }
}

/// An identity function. Returns the input string unchanged.
fn identity(s: &str) -> String {
    s.to_string()
}

/// A shift cipher.
///
/// The cipher shifts each letter by 13. It is essentially a Caeser cipher but with a fixed shift.
fn rot13(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());

    for b in s.bytes() {
        if b.is_ascii_alphabetic() {
            out.push(match_case(
                (b.to_ascii_uppercase() - b'A' + 13) % 26 + b'A',
                b,
            ));
        } else {
            out.push(b);
        }
    }

    String::from_utf8(out).unwrap()
}

/// A monoalphabetic substitution cipher.
///
/// The cipher uniquely maps each letter in the alphabet to a different letter in the alphabet.
/// This mapping is then used to map the input string to the output string.
fn aristocrat<R>(s: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let mut out = Vec::with_capacity(s.len());
    let mut mapping = ALPHABET;
    mapping.shuffle(rng);

    for b in s.bytes() {
        if b.is_ascii_alphabetic() {
            out.push(match_case(
                mapping[(b.to_ascii_uppercase() - b'A') as usize],
                b,
            ));
        } else {
            out.push(b);
        }
    }

    String::from_utf8(out).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_TEXT: &str =
        "abcdefghijklmnopqrstuvwxyz 0123456789-!'\".ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    mod mock_rng;
    use mock_rng::MockRng;

    #[test]
    fn test_rot13() {
        let res = rot13(TEST_TEXT);

        assert_eq!(
            res,
            "nopqrstuvwxyzabcdefghijklm 0123456789-!'\".NOPQRSTUVWXYZABCDEFGHIJKLM"
        );
    }

    #[test]
    fn test_aristocrat() {
        let res = aristocrat(TEST_TEXT, &mut MockRng);
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }

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
}
