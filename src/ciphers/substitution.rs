//! Definition of subtitution ciphers.
//!
//! [`rot13`], [`caeser`], [`aristocrat`]

use super::{match_case, shift_letter, ALPHABET};
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


    let mut shift = rng.next_u32();
    while shift == 0 {
        shift = rng.next_u32();
    }

    let shift = loop {
        let x = rng.next_u32();
        if x != 0 {
            break x
        }
    } as u8;

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

    let mut mapping = Vec::with_capacity(ALPHABET.len());
    let mut alphabet = ALPHABET.to_vec();
    while !alphabet.is_empty() {
        let i = rng.gen_range(0..alphabet.len());
        // if letter maps to itself, try again
        if alphabet[i] - b'a' == mapping.len() as u8 {
            continue
        }
        mapping.push(alphabet[i]);
        alphabet.swap_remove(i);
    }

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
    use crate::ciphers::tests::MockRng;

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
        let res = caeser(TEST_TEXT, &mut MockRng::new());
        assert_eq!(res, TEST_TEXT)
    }

    #[test]
    fn test_aristocrat() {
        let res = aristocrat(TEST_TEXT, &mut MockRng::new());
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }
}
