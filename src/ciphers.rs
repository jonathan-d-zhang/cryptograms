#[derive(GraphQLEnum)]
pub enum Type {
    Rot13,
}

#[derive(GraphQLEnum)]
pub enum Length {
    Short,
    Medium,
    Long,
}

#[derive(GraphQLObject)]
pub struct Cipher {
    #[graphql(skip)]
    /// The unencrypted text.
    plaintext: String,
    /// The encrypted text.
    ciphertext: String,
    /// The type of cipher used.
    r#type: Type,
    /// The length of the plaintext.
    length: Length,
}

impl Cipher {
    pub fn new(plaintext: &str) -> Self {
        Cipher {
            plaintext: plaintext.to_string(),
            ciphertext: String::new(),
            r#type: Type::Rot13,
            length: Length::Short,
        }
    }
}

struct Rot13;

fn rot13(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    for chr in s.chars() {
        let offset;
        if chr.is_ascii_uppercase() {
            offset = b'A';
        } else {
            offset = b'a';
        }
        out.push(((chr as u8 - offset + 13) % 26 + offset) as char)
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_TEXT: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn test_rot13() {
        let res = rot13(TEST_TEXT);

        assert_eq!(res, "nopqrstuvwxyzabcdefghijklmNOPQRSTUVWXYZABCDEFGHIJKLM");
    }
}
