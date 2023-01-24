//! Contains the two morse code ciphers, Morbit and Pollux.

mod morbit;
pub use morbit::morbit;

/// Morse alphabet.
const MORSE_ALPHABET: [&str; 26] = [
    ".-", "-...", "-.-.", "-..", ".", "..-.", "--.", "....", "..", ".---", "-.-", ".-..", "--",
    "-.", "---", ".--.", "--.-", ".-.", "...", "-", "..-", "...-", ".--", "-..-", "-.--", "--..",
];

/// Encode an ascii letter in morse code.
///
/// Currently only supports letters.
fn morse_encode(b: u8) -> String {
    if b.is_ascii_alphabetic() {
        let index = b.to_ascii_lowercase() - b'a';

        MORSE_ALPHABET[index as usize].into()
    } else {
        panic!("Can only morse encode ascii letters");
    }
}
