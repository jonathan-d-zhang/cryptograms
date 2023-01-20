//! Definition of subtitution ciphers.
//!
//! [`rot13`], [`caeser`], [`aristocrat`], [`patristocrat`]

use super::{match_case, shift_letter, ALPHABET};
use rand::prelude::*;

const ROT13_MAPPING: &[u8] = b"nopqrstuvwxyzabcdefghijklm";

/// Generic function that implements the various substitution ciphers
fn substitute(s: &str, mapping: &[u8], keep_whitespace: bool) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if !keep_whitespace && b.is_ascii_whitespace() {
            continue;
        }

        if b.is_ascii_alphabetic() {
            out.push(match_case(mapping[(b.to_ascii_uppercase() - b'A') as usize], b) as char);
        } else {
            out.push(b as char);
        }
    }

    out
}

/// Shift each letter by 13.
///
/// The cipher shifts each letter by 13. It is essentially a Caeser cipher but with a fixed shift.
pub fn rot13(s: &str) -> String {
    substitute(s, ROT13_MAPPING, true)
}

/// Randomly choose a shift `s` and shift each letter by `s`.
pub fn caeser<R>(s: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let shift = loop {
        let x = rng.next_u32();
        if x != 0 {
            break x;
        }
    } as u8;

    let mut mapping = [0u8; 26];
    for (i, &b) in ALPHABET.iter().enumerate() {
        mapping[i] = shift_letter(b, shift)
    }

    substitute(s, &mapping, true)
}

/// Monoalphabetic substitution cipher.
///
/// The cipher uniquely maps each letter in the alphabet to a different letter in the alphabet.
/// This mapping is then used to map the input string to the output string.
pub fn aristocrat<R>(s: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let mut mapping: Vec<_> = ALPHABET.choose_multiple(rng, 26).copied().collect();
    loop {
        if (mapping.iter().zip(ALPHABET.iter())).all(|p| p.0 != p.1) {
            break;
        }
        mapping.shuffle(rng);
    }

    substitute(s, &mapping, true)
}

/// Similar to aristocrat, but removes all spaces.
pub fn patristocrat<R>(s: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let mut mapping: Vec<_> = ALPHABET.choose_multiple(rng, 26).copied().collect();
    loop {
        if (mapping.iter().zip(ALPHABET.iter())).all(|p| p.0 != p.1) {
            break;
        }
        mapping.shuffle(rng);
    }

    substitute(s, &mapping, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

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
        let res = caeser(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }

    #[test]
    fn test_aristocrat() {
        let res = aristocrat(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }
    #[test]
    fn test_patristocrat() {
        let res = patristocrat(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans = "bcdefghijklmnopqrstuvwxyza0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res, ans);
    }
}
