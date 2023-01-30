//! Contains the two morse code ciphers, Morbit and Pollux.

pub(super) mod morbit;
pub(super) use morbit::morbit;

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

#[cfg(test)]
mod tests {
    use super::{morse_encode, MORSE_ALPHABET};

    #[test]
    fn test_morse_encode() {
        for b in b'a'..=b'z' {
            assert_eq!(MORSE_ALPHABET[(b - b'a') as usize], morse_encode(b));
        }
    }
}
