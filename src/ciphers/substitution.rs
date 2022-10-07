//! Definition of subtitution ciphers.
//!
//! [`rot13`], [`caeser`], [`aristocrat`]

use super::{shift_letter, match_case, ALPHABET};
use rand::prelude::*;

/// Shift each letter by 13.
///
/// The cipher shifts each letter by 13. It is essentially a Caeser cipher but with a fixed shift.
pub fn rot13(s: &str) -> String {
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
pub fn caeser<R>(s: &str, rng: &mut R) -> String
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
pub fn aristocrat<R>(s: &str, rng: &mut R) -> String
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
    use crate::ciphers::tests::MockRng;
    use super::*;

    static TEST_TEXT: &str =
        "abcdefghijklmnopqrstuvwxyz 0123456789-!'\".ABCDEFGHIJKLMNOPQRSTUVWXYZ";

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
        // The mock shift is 0, so it's the same as an identity function.
        let res = caeser(TEST_TEXT, &mut MockRng);
        assert_eq!(res, TEST_TEXT)
    }

    #[test]
    fn test_aristocrat() {
        let res = aristocrat(TEST_TEXT, &mut MockRng);
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }
}
