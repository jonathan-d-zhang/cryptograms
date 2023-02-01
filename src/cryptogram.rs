//! This module defines the Cryptogram object for the public interface.

use super::ciphers::{Cipher, CipherResult};
use super::quotes;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Describe the type of cipher used to encrypt a [`Cryptogram`]
///
/// Each of the variants has an accompanying function with a lowercased name.
/// For example, [`Type::Identity`] has the function [`crate::ciphers::identity`].
#[derive(GraphQLEnum, Debug, Copy, Clone)]
pub enum Type {
    /// Returns the plaintext unchanged. See [`crate::ciphers::identity`] for more details.
    Identity,
    /// Shift letters by 13. See [`crate::ciphers::substitution::rot13`] for more details.
    Rot13,
    /// Shift letters by a random amount. See [`crate::ciphers::substitution::caeser`] for more details.
    Caesar,
    /// Monoalphabetic substitution. See [`crate::ciphers::substitution::aristocrat`] for more details.
    Aristocrat,
    /// Monoalphabetic substitution, spaces ignored. See [`crate::ciphers::substitution::patristocrat`] for more details.
    Patristocrat,
    /// Monoalphabetic substitution, spaces ignored, keyed plaintext alphabet. See
    /// [`crate::ciphers::substitution::patristocrat_k1`] for more details.
    PatristocratK1,
    /// Monoalphabetic substitution, spaces ignored, keyed ciphertext alphabet. See
    /// [`crate::ciphers::substitution::patristocrat_k1`] for more details.
    PatristocratK2,
    Pollux,
    Morbit,
    // Too unoptimized for now
    //    Cryptarithm,
    /// Polyalphabetic substitution, spaces ignored. See ['crate::ciphers::hill`] for more details.
    Hill,
    // TODO: Add xenocrypt
}

/// The length of a cipher.
///
/// The ranges for each variant are start inclusive and end exclusive.
#[derive(GraphQLEnum, Debug, Copy, Clone)]
pub enum Length {
    /// Quotations ranging from 60 to 90 bytes.
    Short,
    /// Quotations ranging from 90 to 120 bytes.
    Medium,
    /// Quotations ranging from 120 to 150 bytes.
    Long,
}

#[derive(GraphQLObject)]
pub struct Answer {
    /// The plaintext
    pub plaintext: String,

    /// The key used to encrypt, if applicable.
    pub key: Option<String>,
}

impl Answer {
    #[must_use]
    pub fn new(plaintext: String, key: Option<String>) -> Self {
        Self { plaintext, key }
    }
}

#[derive(GraphQLObject)]
pub struct Cryptogram {
    /// The encrypted text.
    pub ciphertext: String,
    /// The type of cipher used.
    pub r#type: Type,
    /// The length of the plaintext.
    pub length: Length,
    /// The author of the quote.
    pub author: Option<String>,

    /// Token to request the plaintext.
    pub token: i32,

    /// The key used to encrypt, if applicable.
    #[graphql(skip)]
    pub key: Option<String>,

    /// The plaintext
    #[graphql(skip)]
    pub plaintext: String,

    /// Character frequencies, if applicable
    pub frequencies: Option<Vec<i32>>,
}

impl Cryptogram {
    /// Create a Cryptogram from plaintext, length, and type
    ///
    /// If plaintext is not given, then a random quotation is selected.
    /// The default `length` is [`Length::Medium`] and the default `r#type`
    /// is [`Type::Identity`], though this may change in the future.
    pub(crate) fn new(
        plaintext: Option<String>,
        length: Option<Length>,
        r#type: Option<Type>,
        key: Option<String>,
    ) -> CipherResult<Self> {
        use Type::{Aristocrat, Caesar, Identity, Patristocrat, PatristocratK1, PatristocratK2};
        let r#type = r#type.unwrap_or(Identity);

        let length = length.unwrap_or(Length::Medium);

        let quote = match plaintext {
            Some(t) => quotes::Quote::new(t, None),
            None => quotes::fetch_quote(length),
        };

        let cipher = Cipher::encrypt(&quote.text, r#type, key)?;

        let frequencies = match r#type {
            Identity | Caesar | Aristocrat | Patristocrat | PatristocratK1 | PatristocratK2 => {
                Some(frequencies(&cipher.ciphertext))
            }
            _ => None,
        };

        Ok(Self {
            ciphertext: cipher.ciphertext.to_uppercase(),
            r#type,
            length,
            author: quote.author,
            token: compute_hash(&cipher.ciphertext),
            key: cipher.key,
            plaintext: quote.text,
            frequencies,
        })
    }
}

fn frequencies(s: &str) -> Vec<i32> {
    let mut freqs = vec![0; 26];
    for b in s.to_uppercase().bytes() {
        if b.is_ascii_alphabetic() {
            freqs[(b - b'A') as usize] += 1;
        }
    }

    freqs
}

fn compute_hash(s: &str) -> i32 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as i32
}

#[cfg(test)]
mod tests {
    use super::frequencies;

    #[test]
    fn test_frequencies() {
        let input = "aaaaabbccac";
        let output = frequencies(input);
        let mut ans = vec![0; 26];
        ans[0] = input.chars().filter(|c| *c == 'a').count() as i32;
        ans[1] = input.chars().filter(|c| *c == 'b').count() as i32;
        ans[2] = input.chars().filter(|c| *c == 'c').count() as i32;

        assert_eq!(output, ans);
    }
}
