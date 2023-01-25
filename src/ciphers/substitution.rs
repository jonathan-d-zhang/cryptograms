//! Definition of subtitution ciphers.
//!
//! [`rot13`], [`caeser`], [`aristocrat`], [`patristocrat`]

use super::CipherResult;
use super::{match_case, shift_letter, Cipher, ALPHABET, WORDS};
use itertools::Itertools;
use rand::prelude::*;

const ROT13_MAPPING: [u8; 26] = *b"nopqrstuvwxyzabcdefghijklm";

/// Generic function that implements the various substitution ciphers
///
/// `mapping` maps letters from plaintext and ciphertext. For example, a b'e' in the 0th index
/// would mean that b'a' maps to b'e'.
fn substitute(s: &str, mapping: &[u8], keep_whitespace: bool) -> String {
    let mut out = String::with_capacity(s.len());
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

    if !keep_whitespace {
        let mut interspersed = String::with_capacity(s.len() + s.len() / 5);
        for chunk in &out.chars().chunks(5) {
            chunk.for_each(|c| interspersed.push(c));
            interspersed.push(' ');
        }
        interspersed.pop();

        interspersed
    } else {
        out
    }
}

/// Shift each letter by 13.
///
/// The cipher shifts each letter by 13. It is essentially a Caeser cipher but with a fixed shift.
pub(super) fn rot13(s: &str) -> CipherResult<Cipher> {
    Ok(Cipher::new(substitute(s, &ROT13_MAPPING, true), None))
}

/// Randomly choose a shift `s` and shift each letter by `s`.
pub(super) fn caeser<R>(s: &str, rng: &mut R) -> CipherResult<Cipher>
where
    R: Rng + ?Sized,
{
    let shift = loop {
        let x = rng.next_u32();
        if x != 0 {
            break x;
        }
    } as u8;

    let mut mapping = [0u8; 255];
    for b in ALPHABET {
        mapping[(b - b'a') as usize] = shift_letter(b, shift);
    }

    Ok(Cipher::new(substitute(s, &mapping, true), None))
}

/// Monoalphabetic substitution cipher.
///
/// The cipher uniquely maps each letter in the alphabet to a different letter in the alphabet.
/// This mapping is then used to map the input string to the output string.
pub(super) fn aristocrat<R>(s: &str, rng: &mut R) -> CipherResult<Cipher>
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

    Ok(Cipher::new(substitute(s, &mapping, true), None))
}

/// Similar to aristocrat, but removes all spaces.
pub(super) fn patristocrat<R>(s: &str, rng: &mut R) -> CipherResult<Cipher>
where
    R: Rng + ?Sized,
{
    let mut mapping = ALPHABET.choose_multiple(rng, 26).copied().collect_vec();
    loop {
        if (mapping.iter().zip(ALPHABET.iter())).all(|p| p.0 != p.1) {
            break;
        }
        mapping.shuffle(rng);
    }

    Ok(Cipher::new(substitute(s, &mapping, false), None))
}

/// Essentially the same as patristocrat, but uses a keyed plaintext.
pub(super) fn patristocrat_k1<R>(s: &str, key: Option<String>, rng: &mut R) -> CipherResult<Cipher>
where
    R: Rng + ?Sized,
{
    // the final mapping we're trying to create contains a key in the plaintext alphabet, mapped to
    // the ciphertext. This looks something like
    // samplekybcdfghijnoqrtuvwxz
    // ABCDEFGHIJKLMNOPQRSTUVWXYZ
    // In this case, our key is "samplekey".
    let mut key_chars = [false; 255];

    let mut mapping = [0u8; 26];
    let mut i = 0;

    // mapping starts with the key, which is either randomly chosen or given
    // unwrap is safe because WORDS is guaranteed non-empty
    let key = key
        .as_ref()
        .unwrap_or_else(|| WORDS.choose(rng).unwrap())
        .to_lowercase();

    for b in key.bytes() {
        // letters can only be used once
        if key_chars[b as usize] {
            continue;
        }
        mapping[i] = b;
        key_chars[b as usize] = true;
        i += 1;
    }

    // next, the letters not in the key are appended in alphabetical order
    for b in ALPHABET {
        if !key_chars[b as usize] {
            mapping[i] = b;
            i += 1;
        }
    }

    // finally, shift the plaintext until no letter maps to itself
    // note: we don't shuffle like in `patristocrat` because we must keep the key in place
    loop {
        if (mapping.iter().zip(ALPHABET.iter())).all(|p| p.0 != p.1) {
            break;
        }
        mapping.rotate_right(1);
    }

    Ok(Cipher::new(substitute(s, &mapping, false), Some(key)))
}

/// Essentially the same as patristocrat, but uses a keyed ciphertext.
pub(super) fn patristocrat_k2<R>(s: &str, key: Option<String>, rng: &mut R) -> CipherResult<Cipher>
where
    R: Rng + ?Sized,
{
    // In k2, the ciphertext contains the key. For example, with the key "SAMPLEKEY", our mapping
    // without shifts would be
    // abcdefghijklmnopqrstuvwxyz
    // SAMPLEKYBCDFGHIJNOQRTUVWXZ
    // we would then shift the ciphertext until no letters map to themselves. This is almost
    // identical to k1 patristocrat.
    let mut key_chars = [false; 255];

    let mut mapping = [0u8; 26];

    // unwrap is safe because WORDS is guaranteed non-empty
    let key = key
        .as_ref()
        .unwrap_or_else(|| WORDS.choose(rng).unwrap())
        .to_lowercase();

    let mut i = 0;

    // first, map the first `key.len()` plaintext chars to the key.
    for b in key.bytes() {
        if key_chars[b as usize] {
            continue;
        }
        mapping[i] = b;
        key_chars[b as usize] = true;
        i += 1;
    }

    // next, we map the rest of the plaintext to the rest of the ciphertext alphabet
    for b in ALPHABET.into_iter() {
        if !key_chars[b as usize] {
            mapping[i] = b;
            i += 1;
        }
    }

    // finally, shift the mapping until no letter maps to itself
    // note: we don't shuffle like in patristocrat because we must keep the key in place
    loop {
        if (mapping.iter().zip(ALPHABET.iter())).all(|p| p.0 != p.1) {
            break;
        }
        mapping.rotate_left(1);
    }

    Ok(Cipher::new(substitute(s, &mapping, false), Some(key)))
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
            res.unwrap().ciphertext,
            "nopqrstuvwxyzabcdefghijklm 0123456789-!'\".NOPQRSTUVWXYZABCDEFGHIJKLM"
        );
    }

    #[test]
    fn test_caesar() {
        let res = caeser(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res.unwrap().ciphertext, ans);
    }

    #[test]
    fn test_aristocrat() {
        let res = aristocrat(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans = "bcdefghijklmnopqrstuvwxyza 0123456789-!'\".BCDEFGHIJKLMNOPQRSTUVWXYZA";
        assert_eq!(res.unwrap().ciphertext, ans);
    }
    #[test]
    fn test_patristocrat() {
        let res = patristocrat(TEST_TEXT, &mut StepRng::new(0, 1));
        let ans =
            "bcdef ghijk lmnop qrstu vwxyz a0123 45678 9-!'\" .BCDE FGHIJ KLMNO PQRST UVWXY ZA";
        assert_eq!(res.unwrap().ciphertext, ans);
    }

    #[test]
    fn test_patristocrat_k1() {
        let res = patristocrat_k1(
            "bcdefghijklmnopqrstuvwxyza",
            Some(String::from("testkey")),
            &mut StepRng::new(0, 1),
        );
        let ans = "tesky abcdf ghijl mnopq ruvwx z";

        assert_eq!(res.unwrap().ciphertext, ans);
    }

    #[test]
    fn test_patristocrat_k2() {
        let res = patristocrat_k2(
            "yzabcdefghijklmnopqrstuvwx",
            Some(String::from("testkey")),
            &mut StepRng::new(0, 1),
        );
        let ans = "tesky abcdf ghijl mnopq ruvwx z";

        assert_eq!(res.unwrap().ciphertext, ans);
    }
}
