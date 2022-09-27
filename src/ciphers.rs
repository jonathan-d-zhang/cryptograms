use super::quotes;
use rand::prelude::*;

const ALPHABET: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";

#[derive(GraphQLEnum, Copy, Clone)]
pub enum Type {
    /// Returns the plaintext unchanged.
    Identity,
    /// Shift letters by 13.
    Rot13,
    /// Monoalphabetic substitution
    Aristocrat,
}

#[derive(GraphQLEnum, Copy, Clone)]
pub enum Length {
    Short,
    Medium,
    Long,
}

#[derive(GraphQLObject)]
pub struct Cryptogram {
    /// The encrypted text.
    ciphertext: String,
    #[graphql(skip)]
    /// The unencrypted text.
    plaintext: String,
    #[graphql(skip)]
    /// The type of cipher used.
    r#type: Type,
    #[graphql(skip)]
    /// The length of the plaintext.
    length: Length,
    /// The author of the quote.
    author: Option<String>,
}

impl Cryptogram {
    pub fn new(plaintext: Option<String>, length: Option<Length>, r#type: Option<Type>) -> Self {
        use Type::*;
        let r#type = r#type.unwrap_or_else(|| Identity);

        let cipher = match r#type {
            Identity => identity,
            Rot13 => rot13,
            Aristocrat => aristocrat,
        };

        let length = length.unwrap_or_else(|| Length::Medium);

        let quote = match plaintext {
            Some(t) => quotes::Quote::new(t, None),
            None => quotes::fetch_quote(length),
        };

        let ciphertext = cipher(&quote.text, &mut thread_rng());

        Self {
            plaintext: quote.text,
            ciphertext,
            r#type,
            length,
            author: quote.author,
        }
    }
}

const fn ord(chr: char) -> u8 {
    chr as u8
}

const fn chr(ord: u8) -> char {
    ord as char
}

/// Adjust the case of ord to match the case of to_match
fn match_case(ord: u8, to_match: u8) -> u8 {
    let is_lower = (to_match >> 5) & 1;
    ord & !(1 << 5) | (is_lower << 5)
}

fn identity<R: Rng + ?Sized>(s: &str, _: &mut R) -> String {
    s.to_string()
}

fn rot13<R: Rng + ?Sized>(s: &str, _: &mut R) -> String {
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

    static TEST_TEXT: &'static str =
        "abcdefghijklmnopqrstuvwxyz 0123456789-!'\".ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    mod mock_rng;
    use mock_rng::MockRng;

    #[test]
    fn test_rot13() {
        let res = rot13(TEST_TEXT, &mut MockRng);

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
