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
struct Cipher {
    /// The encrypted text.
    ciphertext: String,
    #[graphql(skip)]
    /// The unencrypted text.
    plaintext: String,
    /// The type of cipher used.
    r#type: Type,
    /// The length of the plaintext.
    length: Length,
}

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
