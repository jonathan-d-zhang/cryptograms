//! Cryptarithm generator.
//!
//! A cryptarithm, or alphametic, is a puzzle in which you are given a mathematical equation where
//! the numbers are represented by letters. The canonical example is "SEND + MORE = MONEY". The
//! solution is O=0, M=1, Y=2, E=5, N=6, D=7, R=8, S=9.
//!
//! See [this](https://en.wikipedia.org/wiki/Verbal_arithmetic) for more info.
#![allow(dead_code)]

use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering::Relaxed},
    Mutex,
};

use super::WORDS;
use super::{Cipher, CipherResult};

static mut STATS: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];

fn word_to_int(word: &str, mapping: &HashMap<char, i32>) -> i32 {
    let mut r = 0;
    for chr in word.chars() {
        r = r * 10 + mapping[&chr];
    }

    r
}

#[derive(Debug)]
struct Pattern {
    length: usize,
    pattern: Regex,
    used_letters: HashSet<char>,
    indexes: HashMap<char, Vec<usize>>,
}

impl Pattern {
    pub fn new(num: i32, mapping: &HashMap<char, i32>) -> Self {
        let reverse_mapping: HashMap<char, char> = mapping
            .iter()
            .map(|(&k, &v)| ((v + 48) as u8 as char, k))
            .collect();
        log::trace!("Generated reverse_mapping={:?}", reverse_mapping);

        let mut pat = String::new();
        let mut indexes = HashMap::new();
        let assigned_chars: String = mapping.keys().collect();
        let allowed_wild_chars = format!("[^{assigned_chars}]");

        for (i, chr) in num.to_string().chars().enumerate() {
            if let Some(&n) = reverse_mapping.get(&chr) {
                pat.push(n);
            } else {
                indexes.entry(chr).or_insert_with(Vec::new).push(i);
                pat.push_str(&allowed_wild_chars);
            }
        }
        log::trace!("Created pattern {pat:?} from {num}");
        Self {
            length: (num as f64).log10().ceil() as usize,
            pattern: Regex::new(&format!("^{pat}$")).unwrap(),
            used_letters: mapping.keys().copied().collect(),
            indexes,
        }
    }

    // TODO: extract regex logic so it's testable
    fn gen_regex() -> String {
        String::new()
    }

    fn matches_pattern(&self, word: &str) -> bool {
        if word.len() != self.length {
            unsafe {
                STATS[0].fetch_add(1, Relaxed);
            }
            return false;
        }

        if !self.pattern.is_match(word) {
            unsafe {
                STATS[1].fetch_add(1, Relaxed);
            }
            return false;
        }

        // characters at indexes within a key must be equal
        // characters at indexes across keys must be different
        if !self.indexes.is_empty() {
            let bytes = word.as_bytes();

            let mut seen = self.used_letters.clone();
            for v in self.indexes.values() {
                if v.len() > 1 {
                    let letters: HashSet<_> = v.iter().map(|&i| bytes[i]).collect();

                    if letters.len() != 1 {
                        unsafe {
                            STATS[2].fetch_add(1, Relaxed);
                        }
                        return false;
                    }
                }

                // fail if we've already seen this char
                if !seen.insert(bytes[v[0]] as char) {
                    unsafe {
                        STATS[2].fetch_add(1, Relaxed);
                    }
                    return false;
                }
            }
        }

        true
    }
}

/// Returns the word that would form a valid cryptarithm if possible
fn create_cryptarithm(a: &str, b: &str, words: &[&String]) -> Option<String> {
    log::debug!("Checking a={a} b={b}");
    // map each letter to a number
    let unique_letters: HashSet<char> = a.chars().chain(b.chars()).collect();
    let unique_letter_count = unique_letters.len();

    // too many letters for a unique mapping
    if unique_letter_count > 10 {
        return None;
    }

    let n = a.len();
    let m = b.len();

    let length_range = n.max(m)..=n.max(m) + 1;
    // filter out a, b, and words that are mathematically impossible
    let filtered: Vec<_> = words
        .iter()
        .filter(|&&w| length_range.contains(&w.len()) && w != a && w != b)
        .collect();

    if filtered.is_empty() {
        log::debug!("Filtered all words, returning");
        return None;
    }

    log::debug!("Searching in {:?}", filtered);

    let solutions = Mutex::new(Vec::new());
    for (i, p) in (0..=9).permutations(unique_letter_count).enumerate() {
        if i % 100000 == 0 {
            unsafe {
                log::debug!("Tries: {:?}", STATS);
            }
        }

        // map unique letters to digits
        let mapping: HashMap<char, i32> = unique_letters
            .iter()
            .zip(p.iter())
            .map(|(&k, &v)| (k, v))
            .collect();
        log::trace!("Generated mapping={:?}", mapping);

        let num_a = word_to_int(a, &mapping);
        let num_b = word_to_int(b, &mapping);

        // if there are leading zeros, skip
        if mapping[&a.chars().next().unwrap()] == 0 || mapping[&b.chars().next().unwrap()] == 0 {
            log::trace!("Skip: leading zero");
            continue;
        }

        log::trace!("Mapped {a:?} to {num_a} and {b:?} to {num_b}");

        // check what words match the pattern of numbers in the sum
        let s = num_a + num_b;
        let pat = Pattern::new(s, &mapping);

        /*        for word in words.iter().filter(|&&word| word != a && word != b) {
                    if length_range.start > word.len() || word.len() > length_range.end {
                        continue
                    }

                    if pat.matches_pattern(word) {
                        let reverse_mapping: HashMap<char, char> = mapping.iter().map(|(&k, &v)| ((v + 48) as u8 as char, k)).collect();
        //                log::debug!("Mapping={mapping:?}");
        //                log::debug!("Reverse mapping={reverse_mapping:?}");
        //                log::debug!("Pattern={pat:?}");
        //                println!("Solution found: {word}");

                       if !solutions.is_empty() {
                            return None
                        }
                        solutions.push((num_a, num_b, s, word));
                    }
                }
        */
        filtered
            .par_iter()
            .filter(|&&&word| pat.matches_pattern(word))
            .for_each(|w| solutions.lock().unwrap().push(w));
    }

    let sol = solutions.into_inner().unwrap();
    if sol.len() == 1 {
        //        let (num_a, num_b, s, word) = solutions[0];
        //        log::debug!("Solution found: {num_a} + {num_b} = {s}");
        return Some(sol[0].to_string());
    }

    None
}

/// Generates a cryptarithm
///
/// See module level docs for more info about cryptarithms
pub(super) fn cryptarithm<R: Rng + ?Sized>(rng: &mut R) -> CipherResult<Cipher> {
    loop {
        let words: Vec<&String> = WORDS.choose_multiple(rng, 10).collect();
        log::debug!("Words in this batch: {:?}", words);

        // TODO: parallelize
        for i in 0..words.len() {
            for j in i + 1..words.len() {
                let a = words[i];
                let b = words[j];

                log::trace!("Selected {a} and {b}");

                if let Some(c) = create_cryptarithm(a, b, &words) {
                    let cryptarithm = format!("{a} + {b} = {c}");
                    log::info!("Found cryptarithm: {cryptarithm}");
                    unsafe {
                        log::debug!("Tries: {:?}", STATS);
                    }
                    return Ok(Cipher::new(cryptarithm, None));
                }
            }
        }
        unsafe {
            log::debug!("Tries: {:?}", STATS);
        }
        log::debug!("Switching batch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
        #[test]
        fn test_create_cryptarithm() {
            let a = "send";
            let b = "more";

            let x = String::from("send");
            let y = String::from("more");
            let z = String::from("money");
            let words: Vec<&String> = vec![&x, &y, &z];

            assert_eq!(
                create_cryptarithm(a, b, &words),
                Some(String::from("money"))
            );
        }
    */
    #[test]
    fn test_word_to_int() {
        let mapping = HashMap::from([('b', 3), ('c', 5)]);

        assert_eq!(word_to_int("b", &mapping), 3);
        assert_eq!(word_to_int("bc", &mapping), 35);
    }

    #[test]
    fn test_pattern() {}

    #[test]
    fn test_matches_pattern() {
        let pattern = Pattern::new(112233, &HashMap::from([('a', 1), ('b', 2), ('c', 3)]));

        let word = "aabbcc";

        assert!(pattern.matches_pattern(word));
    }
}
