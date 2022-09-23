use super::quotes;

#[derive(GraphQLEnum, Copy, Clone)]
pub enum Type {
    /// Returns the plaintext unchanged.
    Identity,
    /// Shift letters by 13.
    Rot13,
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
        };

        let length = length.unwrap_or_else(|| Length::Medium);

        let quote = match plaintext {
            Some(t) => quotes::Quote::new(t, None),
            None => quotes::fetch_quote(length),
        };

        let ciphertext = cipher(&quote.text);

        Self {
            plaintext: quote.text,
            ciphertext,
            r#type,
            length,
            author: quote.author,
        }
    }
}

fn identity(s: &str) -> String {
    s.to_string()
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
