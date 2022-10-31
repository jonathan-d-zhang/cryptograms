use lazy_static::lazy_static;
use rand::prelude::*;

lazy_static! {
    static ref WORDS: Vec<String> = {
        let words_file = std::env::var("WORDS_FILE").expect("Environment variable WORDS_FILE must be set.");

        log::info!("Loading cryptarithm words from {:?}", words_file);
        let line = std::fs::read_to_string(words_file).unwrap();

        line.split(',').map(str::to_string).collect()
    };
}

fn cryptarithm() -> String {
    // only operate on a randomly chosen subsequence of the word list to reduce computations,
    // reselecting a subsequence if no valid cryptarithm is found

    // arbitrarily chosen
    let n = 50;

    let subseq = WORDS.choose_multiple(&mut thread_rng(), n);

    for i in 0..n {
        for j in i+1..n {

        }
    }

    panic!("No valid cryptarithm")
}

#[cfg(test)]
mod tests {}
