//! This module contains the implementation of the ciphers.

#![warn(missing_docs)]

use super::cryptogram::Type;
use super::cryptogram::Type::*;
use rand::prelude::*;

/// Lowercase alphabet.
const ALPHABET: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";

/// Adjust the case of ord to match the case of to_match
const fn match_case(ord: u8, to_match: u8) -> u8 {
    let is_lower = (to_match >> 5) & 1;
    ord & !(1 << 5) | (is_lower << 5)
}

/// Wrapper function to call a specific cipher by [`Type`].
pub fn encrypt(plaintext: &str, cipher_type: Type) -> String {
    let mut rng = thread_rng();
    match cipher_type {
        Identity => identity(plaintext),
        Rot13 => rot13(plaintext),
        Caesar => caeser(plaintext, &mut rng),
        Aristocrat => aristocrat(plaintext, &mut rng),
    }
}

/// Returns the input string unchanged.
fn identity(s: &str) -> String {
    s.to_string()
}

/// Convenience function to shift `b` by `by` places.
const fn shift_letter(b: u8, by: u8) -> u8 {
    let offset = match_case(b'a', b);

    (b - offset + by) % 26 + offset
}

/// Shift each letter by 13.
///
/// The cipher shifts each letter by 13. It is essentially a Caeser cipher but with a fixed shift.
fn rot13(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());

    for b in s.bytes() {
        if b.is_ascii_alphabetic() {
            out.push(shift_letter(b, 13));
        } else {
            out.push(b);
        }
    }

    String::from_utf8(out).unwrap()
}

/// Randomly choose a shift `s` and shift each letter by `s`.
fn caeser<R>(s: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let mut out = Vec::with_capacity(s.len());

    let shift = rng.next_u32() as u8;

    for b in s.bytes() {
        if b.is_ascii_alphabetic() {
            out.push(shift_letter(b, shift));
        } else {
            out.push(b);
        }
    }

    String::from_utf8(out).unwrap()
}

/// Monoalphabetic substitution cipher.
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
    fn test_caesar() {
        // The mock shift is 0
        let res = caeser(TEST_TEXT, &mut MockRng);
        assert_eq!(res, TEST_TEXT)
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
