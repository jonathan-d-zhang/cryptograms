//! Hill Cipher
//!
//! This cipher uses matrix multiplication to encrypt and ignores spaces and symbols.
//!
//! Given a plaintext, which can be any length,
//! and a key, whose length must be a perfect square, the encryption is done as follows.
//! First the key is turned into an `NxN` matrix, where `N` is the square root of the length of the
//! key.
//! Then, each substring of `N` elements in the plaintext is multiplied by the key mod 26.
//!
//! If the plaintext is not a multiple of `N`, the plaintext is padded with "z" until it is.
//!
//! For example, if wanted to encrypt the plaintext "abc" with the key "abcd", we would start by
//! first computing `N`. In this case, `N=2`.
//! Next, we pad the plaintext with "z" until it is divisible by `N`.
//! Our resulting string is now "abcz".
//! We also must turn our 1-d key into a 2-d matrix with row and column lengths equal to `N`. In this case, "abcd" results in the key:
//! [
//!     [0, 1],
//!     [2, 3],
//! ].
//! Each substring of length `2` is then matrix multiplied by the key.
//! The first substring is "ab". `key * [[0], [1]] = [1, 3] mod 26`. These are the first 2 characters of the
//! ciphertext. This process is repeated until the entire plaintext is processed.
//! The resulting ciphertext is "bdzb"

use std::cmp::Ordering;

use super::Cipher;
use super::{CipherError, CipherResult, ErrorKind};
use rand::prelude::*;

const KEY_LENGTH: usize = 4;

fn generate_key<R>(rng: &mut R) -> Vec<u8>
where
    R: Rng + ?Sized,
{
    let t: Vec<_> = (b'a'..=b'z').collect();
    t.choose_multiple(rng, KEY_LENGTH).copied().collect()
}

fn matmul(plaintext: &[u8], key: Vec<Vec<u8>>) -> Vec<u8> {
    log::trace!(
        "Matmulling plaintext={:?} with key={:?}",
        String::from_utf8_lossy(plaintext),
        key.clone()
            .into_iter()
            .map(|r| String::from_utf8(r).unwrap())
            .collect::<Vec<_>>()
    );

    // our plaintexts are limited to 160 bytes, naive algo is fine
    let mut result = Vec::new();

    for i in (0..plaintext.len()).step_by(key.len()) {
        for j in 0..key.len() {
            let mut s: u32 = 0;
            for k in 0..key.len() {
                log::trace!("    i={i}, j={j}, k={k}, s={s}");
                s += u32::from(plaintext[i + k] - b'a') * u32::from(key[j][k] - b'a');
            }
            result.push((s % 26) as u8 + b'a');
            log::trace!("    Added {}", ((s % 26) as u8 + b'a') as char);
        }
    }

    log::trace!("    Final Result: {:?}", String::from_utf8(result.clone()));

    result
}

fn is_perfect_square(n: usize) -> bool {
    let mut i = 1;
    loop {
        let t = i * i;
        match t.cmp(&n) {
            Ordering::Equal => return true,
            Ordering::Greater => return false,
            Ordering::Less => i += 1,
        }
    }
}

pub(super) fn hill<R>(plaintext: &str, key: Option<String>, rng: &mut R) -> CipherResult<Cipher>
where
    R: Rng + ?Sized,
{
    let key = match key {
        Some(k) => k.bytes().collect(),
        None => generate_key(rng),
    };

    log::debug!("Hill: key={:?}", String::from_utf8(key.clone()).unwrap());

    let n = key.len();

    // key must be a perfect square
    if !is_perfect_square(n) {
        log::debug!("Key length {} is not a perfect square", n);
        return Err(CipherError::new(
            ErrorKind::KeyError,
            "Key length must be a perfect square".into(),
        ));
    }

    let side_length = (n as f64).sqrt() as usize;

    // remove non-letters
    let mut filtered = String::with_capacity(plaintext.len());
    for c in plaintext.chars() {
        if c.is_ascii_alphabetic() {
            filtered.push(c.to_ascii_lowercase());
        }
    }

    // if the length is not divisible by `side_length`, pad with 'z'
    let to_pad = filtered.len() % side_length;

    filtered.push_str(&"z".repeat(to_pad));

    // convert 1-d key into square matrix
    let mut bytes = key.clone().into_iter();
    let mut matrix = vec![vec![0; side_length]; side_length];
    for row in &mut matrix {
        for item in row.iter_mut() {
            *item = bytes.next().unwrap();
        }
    }

    let r = matmul(filtered.as_bytes(), matrix);

    Ok(Cipher::new(
        String::from_utf8(r).unwrap(),
        Some(String::from_utf8(key).unwrap()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_generate_key() {
        let mut rng = StepRng::new(0, 1);
        let res = generate_key(&mut rng);

        assert_eq!(res, vec![b'x', b'y', b'z', b'a']);
    }

    #[test]
    fn test_hill() {
        let mut rng = StepRng::new(0, 1);
        let res = hill("abcd", Some("abcd".into()), &mut rng).unwrap();

        assert_eq!(res.ciphertext, "bddn");
    }

    #[test]
    fn test_matmul() {
        let plaintext = b"abcd";
        let key = vec![vec![b'a', b'b'], vec![b'c', b'd']];

        let res = matmul(plaintext, key);
        println!("{:?}", String::from_utf8(res.clone()).unwrap());

        assert_eq!(res, vec![b'b', b'd', b'd', b'n'])
    }

    #[test]
    fn test_is_perfect_square() {
        assert!(is_perfect_square(4));
        assert!(is_perfect_square(9));
        assert!(!is_perfect_square(15));
    }
}
