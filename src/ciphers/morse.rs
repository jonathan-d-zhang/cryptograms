//! Definition of the two morse code based ciphers, Morbit and Pollux

use super::MORSE_ALPHABET;

/// Encode an ascii letter in morse code.
///
/// In the future, numbers and punctuation may be supported.
fn morse_encode(b: u8) -> String {
    if b.is_ascii_alphabetic() {
        let index = b.to_ascii_lowercase() - b'a';

        MORSE_ALPHABET[index as usize].into()
    } else {
        panic!("Can only morse encode ascii letters");
    }
}
