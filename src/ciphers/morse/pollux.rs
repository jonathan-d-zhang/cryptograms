//! Defines Pollux Cipher
use super::{super::Cipher, morse_encode};
use itertools::Itertools;
use rand::prelude::*;

// Returns an array with 3 vectors representing the mapping.
// index 0 are the mappings for " ", index 1 is for "-", index 2 for "."
fn make_mapping<R>(rng: &mut R) -> [Vec<char>; 3]
where
    R: Rng + ?Sized,
{
    let mut ret = [Vec::new(), Vec::new(), Vec::new()];
    let mut v = ('0'..='9').collect_vec();
    v.shuffle(rng);

    let mut iter = v.into_iter();
    for mapping in &mut ret {
        for _ in 0..3 {
            mapping.push(iter.next().unwrap());
        }
    }

    ret
}

pub(in super::super) fn pollux<R>(plaintext: &str, rng: &mut R) -> Cipher
where
    R: Rng + ?Sized,
{
    let mapping = make_mapping(rng);
    let mut words: Vec<String> = Vec::new();
    for word in plaintext.split_ascii_whitespace() {
        let mut mapped_word = Vec::new();
        for b in word.bytes() {
            let mut mapped_letters = String::new();
            for morse_byte in morse_encode(b).bytes() {
                mapped_letters.push(match morse_byte {
                    b'-' => *mapping[1].choose(rng).unwrap(),
                    b'.' => *mapping[2].choose(rng).unwrap(),
                    o => o as char,
                });
            }

            mapped_word.push(mapped_letters);
        }

        let n = mapped_word.len();
        words.push(
            mapped_word
                .into_iter()
                .interleave((0..n - 1).map(|_| mapping[0].choose(rng).unwrap().to_string()))
                .collect(),
        );
    }

    let n = words.len();
    Cipher::new(
        words
            .into_iter()
            .interleave((0..n - 1).map(|_| {
                format!(
                    "{}{}",
                    mapping[0].choose(rng).unwrap(),
                    mapping[0].choose(rng).unwrap(),
                )
            }))
            .collect(),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_pollux() {
        let inp = "ee e a";
        let ans = pollux(inp, &mut StepRng::new(0, 1));
        let expected = "7171171174";
        assert_eq!(expected, ans.ciphertext)
    }

    #[test]
    fn test_make_mapping() {
        let map = make_mapping(&mut StepRng::new(0, 1));
        let expected_output = [
            vec!['1', '2', '3'],
            vec!['4', '5', '6'],
            vec!['7', '8', '9'],
        ];
        println!("{map:?}");
        assert_eq!(expected_output, map);
    }
}
