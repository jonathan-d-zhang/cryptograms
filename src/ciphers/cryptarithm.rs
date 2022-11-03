use itertools::Itertools;
use lazy_static::lazy_static;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use regex::Regex;

lazy_static! {
    static ref WORDS: Vec<String> = {
        let words_file =
            std::env::var("WORDS_FILE").expect("Environment variable WORDS_FILE must be set.");

        log::info!("Loading cryptarithm words from {:?}", words_file);
        let line = std::fs::read_to_string(words_file).unwrap();

        line.trim().split(',').filter_map(|s| {
            if 3 < s.len() && s.len() < 8 {
                Some(s.to_string())
            } else {
                None
            }
        }).collect()
    };
}

fn word_to_int(word: &str, mapping: &HashMap<char, i32>) -> i32 {
    let mut r = 0;
    for chr in word.chars() {
        r = r * 10 + mapping[&chr];
    }

    r
}

#[derive(Debug)]
struct Pattern {
    num: i32,
    length: usize,
    pattern: Regex,
    used_letters: HashSet<char>,
    indexes: HashMap<char, Vec<usize>>,
    failed_checks: [u64; 3],
}

impl Pattern {
    pub fn new(num: i32, mapping: &HashMap<char, i32>) -> Self {
        let reverse_mapping: HashMap<char, char> = mapping.iter().map(|(&k, &v)| ((v + 48) as u8 as char, k)).collect();
        log::trace!("Generated reverse_mapping={:?}", reverse_mapping);

        let mut pat = String::new();
        let mut indexes = HashMap::new();
        let assigned_chars: String = mapping.keys().collect();
        let allowed_wild_chars = format!("[^{assigned_chars}]");

        for (i, chr) in num.to_string().chars().enumerate() {
            if let Some(&n) = reverse_mapping.get(&chr) {
                pat.push(n);
            } else {
                indexes.entry(chr).or_insert_with(|| vec![]).push(i);
                pat.push_str(&allowed_wild_chars);
            }
        }
        log::trace!("Created pattern {pat:?} from {num}");
        Self {
            num,
            length: (num as f64).log10().ceil() as usize,
            pattern: Regex::new(&format!("^{pat}$")).unwrap(),
            used_letters: mapping.keys().map(|&chr| chr).collect(),
            indexes,
            failed_checks: [0; 3],
        }
    }

    // TODO: extract regex logic so it's testable
    fn gen_regex() -> String {
        String::new()
    }

    fn matches_pattern(&mut self, word: &str) -> bool {
        if word.len() != self.length {
            self.failed_checks[0] += 1;
            return false;
        }

        if !self.pattern.is_match(word) {
            self.failed_checks[1] += 1;
            return false;
        }

        // characters at indexes within a key must be equal
        // characters at indexes across keys must be different
        if !self.indexes.is_empty() {
            let bytes = word.as_bytes();

            let mut seen = self.used_letters.clone();
            for v in self.indexes.values() {
                if v.len() > 1 {
                    let mut letters: HashSet<_> = v.iter().map(|&i| bytes[i]).collect();

                    if letters.len() != 1 {
                        self.failed_checks[2] += 1;
                        return false
                    }
                }

                // fail if we've already seen this char
                if !seen.insert(bytes[v[0]] as char) {
                    self.failed_checks[2] += 1;
                    return false
                }
            }
        }

        true
    }
}

/// Returns the word that would form a valid cryptarithm if possible
fn create_cryptarithm<'a>(a: &str, b: &str, words: &Vec<&String>) -> Option<String> {
    // map each letter to a number
    let mut unique_letters: HashSet<char> = a.chars().chain(b.chars()).collect();

    let unique_letter_count = unique_letters.len();

    // too many letters for a unique mapping
    if unique_letter_count > 10 {
        return None;
    }

    let mut solutions = Vec::new();
    for p in (0..=9).permutations(unique_letter_count) {
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
            log::trace!("Skip due to leading zero");
            continue
        }

        log::trace!("Mapped {a:?} to {num_a} and {b:?} to {num_b}");

        // check what words match the pattern of numbers in the sum
        let s = num_a + num_b;
        let mut pat = Pattern::new(s, &mapping);

        // TODO: parallelize
        let n = a.len();
        let m = b.len();

        let length_range = n.max(m)..n.max(m)+1;

        for word in words.iter().filter(|&&word| word != a && word != b) {
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
    }

    if solutions.len() == 1 {
        let (num_a, num_b, s, word) = solutions[0];
        log::debug!("Solution found: {num_a} + {num_b} = {s}");
        return Some(word.to_string())
    }

    None
}

pub fn cryptarithm<R: Rng + ?Sized>(rng: &mut R) -> String {
    loop {
        let words: Vec<&String> = WORDS.choose_multiple(rng, 10).collect();
        println!("{:?}", words);

        // TODO: parallelize
        for i in 0..words.len() {
            for j in i + 1..words.len() {
                let a = words[i];
                let b = words[j];

                log::trace!("Selected {a} and {b}");

                if let Some(c) = create_cryptarithm(&a, &b, &words) {
                    let cryptarithm = format!("{a} + {b} = {c}");
                    log::info!("Found cryptarithm: {cryptarithm}");
                    return cryptarithm
                }
            }
        }
        println!("Switching batch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_to_int() {
        let mapping = HashMap::from([('b', 3), ('c', 5)]);

        assert_eq!(word_to_int("b", &mapping), 3);
        assert_eq!(word_to_int("bc", &mapping), 35);
    }

    #[test]
    fn test_pattern() {

    }

    #[test]
    fn test_matches_pattern() {
        let pattern = Pattern::new(
            112233,
            &HashMap::from([('a', 1), ('b', 2), ('c', 3)]),
        );

        let word = "aabbcc";

        assert!(pattern.matches_pattern(word));
    }
}
