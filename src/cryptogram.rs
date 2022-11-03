//! This module defines the Cryptogram object for the public interface.

use super::ciphers::encrypt;
use super::quotes;

/// Describe the type of cipher used to encrypt a [`Cryptogram`]
///
/// Each of the variants should have an accompanying function with a lowercased name.
/// For example, [`Identity`] has the function [`crate::ciphers::identity`].
#[derive(GraphQLEnum, Debug, Copy, Clone)]
pub enum Type {
    /// Returns the plaintext unchanged. See [`crate::ciphers::identity`] for more details.
    Identity,
    /// Shift letters by 13. See [`crate::ciphers::rot13`] for more details.
    Rot13,
    /// Shift letters by a random amount. See [`crate::ciphers::caeser`] for more details.
    Caesar,
    /// Monoalphabetic substitution. See [`crate::ciphers::aristocrat`] for more details.
    Aristocrat,
    Morbit,
    Cryptarithm,
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
    pub token: String,
}

impl Cryptogram {
    /// Create a Cryptogram from plaintext, length, and type
    ///
    /// If plaintext is not given, then a random quotation is selected with
    /// [`crate::quotes::fetch_quote`]. The default `length` is [`Length::Medium`] and the default `r#type`
    /// is [`Type::Identity`], though this may change in the future.
    pub fn new(
        plaintext: Option<String>,
        length: Option<Length>,
        r#type: Option<Type>,
        key: Option<String>,
    ) -> Self {
        use Type::*;
        let r#type = r#type.unwrap_or(Identity);

        let length = length.unwrap_or(Length::Medium);

        let quote = match plaintext {
            Some(t) => quotes::Quote::new(t, None),
            None => quotes::fetch_quote(length),
        };

        let ciphertext = encrypt(&quote.text, r#type, key);

        Self {
            ciphertext,
            r#type,
            length,
            author: quote.author,
            token: "e".into(),
        }
    }
}
