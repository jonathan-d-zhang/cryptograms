use super::Cipher;
use super::WORDS;
use rand::prelude::*;

static TABLEAU: [[u8; 26]; 13] = [
    [
        b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'a', b'b',
        b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm',
    ],
    [
        b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'm', b'a',
        b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    ],
    [
        b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'l', b'm',
        b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k',
    ],
    [
        b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'k', b'l',
        b'm', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j',
    ],
    [
        b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'j', b'k',
        b'l', b'm', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
    ],
    [
        b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b'i', b'j',
        b'k', b'l', b'm', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h',
    ],
    [
        b't', b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b'h', b'i',
        b'j', b'k', b'l', b'm', b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    ],
    [
        b'u', b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'g', b'h',
        b'i', b'j', b'k', b'l', b'm', b'a', b'b', b'c', b'd', b'e', b'f',
    ],
    [
        b'v', b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'f', b'g',
        b'h', b'i', b'j', b'k', b'l', b'm', b'a', b'b', b'c', b'd', b'e',
    ],
    [
        b'w', b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'e', b'f',
        b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'a', b'b', b'c', b'd',
    ],
    [
        b'x', b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'd', b'e',
        b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'a', b'b', b'c',
    ],
    [
        b'y', b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'c', b'd',
        b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'a', b'b',
    ],
    [
        b'z', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'b', b'c',
        b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'a',
    ],
];

pub fn porta<R>(s: &str, key: Option<String>, rng: &mut R) -> Cipher
where
    R: ?Sized + Rng,
{
    let key = key
        .as_ref()
        .unwrap_or_else(|| WORDS.choose(rng).unwrap())
        .to_ascii_lowercase();

    let mut out = Vec::with_capacity(s.len());
    for (mut k, b) in key
        .bytes()
        .cycle()
        .zip(s.to_ascii_lowercase().bytes().filter(|b| b.is_ascii_alphabetic()))
    {
        k -= b'a';
        k -= k & 1;
        k /= 2;
        out.push(TABLEAU[k as usize][(b - b'a') as usize]);
    }

    Cipher::new(String::from_utf8(out).unwrap(), Some(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_porta() {
        let inp = "defendtheeastwallofthecastle";
        let key = "fortification";
        let out = porta(inp, Some(key.into()), &mut StepRng::new(0, 1));

        assert_eq!(out.ciphertext, "synnjscvrnrlahutukucvryrlany");
    }
}
