//! Define the morbit cipher.

use super::{super::Cipher, morse_encode};
use rand::prelude::*;
use std::collections::HashMap;

static MORBIT_BIGRAMS: &[&str] = &["..", ".-", "./", "-.", "--", "-/", "/.", "/-", "//"];

fn generate_key<R>(rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    (0..9)
        .map(|_| ((rng.next_u32() as u8) % 26 + b'a') as char)
        .collect()
}

fn map_key(v: &Vec<u8>) -> Vec<usize> {
    let mut indexes: Vec<_> = (0..v.len()).collect();

    indexes.sort_by_key(|&i| v[i]);

    let mut out = vec![0; 9];
    for (i, &v) in indexes.iter().enumerate() {
        out[v] = i;
    }

    out
}

/// Morbit Cipher
///
/// Morbit is an over-encryption of Morse code, similar to Fractionated Morse Code. The plaintext
/// is first converted into Morse code. Then, pairs of the Morse letters are mapped by the key
/// to the ciphertext.
pub(in super::super) fn morbit(s: &str, key: Option<String>) -> Cipher {
    let key = key
        .unwrap_or_else(|| generate_key(&mut thread_rng()))
        .to_ascii_lowercase();

    // Step 1: Encode the plaintext in Morse code separating chars with "/" and words with "//"

    let mut words = Vec::new();
    let mut chars = Vec::new();
    for word in s.split_ascii_whitespace() {
        for b in word.bytes() {
            if b.is_ascii_alphabetic() {
                chars.push(morse_encode(b));
            }
        }
        words.push(chars.join("/"));
        chars.clear();
    }

    let morse_encoded: Vec<_> = words.join("//").chars().collect();

    // Step 2: Map pairs of Morse symbols to the key
    let bytes = key.bytes().collect();
    let position = map_key(&bytes);
    let mapping: HashMap<_, _> = MORBIT_BIGRAMS
        .iter()
        .zip(position.into_iter())
        .map(|(&b, i)| (b, (i + 1).to_string()))
        .collect();

    let mut out = String::new();

    for pair in morse_encoded.chunks(2) {
        let a = pair[0];
        let b = if pair.len() == 2 { pair[1] } else { '/' };
        out.push_str(&mapping[&format!("{a}{b}") as &str]);
    }

    Cipher::new(out, Some(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_generate_key() {
        let k = generate_key(&mut StepRng::new(0, 1));

        assert_eq!(k, "abcdefghi")
    }

    #[test]
    fn test_map_key() {
        let k = map_key(&"MORSECODE".bytes().collect());

        assert_eq!(k, vec![4, 5, 7, 8, 2, 0, 6, 1, 3]);
    }

    #[test]
    fn test_morbit() {
        let out = morbit("MORE BITS", Some(String::from("MORSECODE")));

        assert_eq!(out.unwrap().ciphertext, "32379749578158");
    }
}
