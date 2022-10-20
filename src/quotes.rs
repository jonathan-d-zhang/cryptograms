use crate::cryptogram::Length;
use juniper::serde::Deserialize;
use lazy_static::lazy_static;
use rand::prelude::*;

lazy_static! {
    static ref QUOTES: Vec<SerQuote> = {
        let quotes_file = std::env::var("QUOTES_FILE");
        let file_contents = std::fs::read_to_string(
            quotes_file
                .as_ref()
                .expect("Environment variable QUOTES_FILE must be set."),
        )
        .unwrap();

        log::info!("Loading quotes from {:?}", quotes_file);
        serde_json::from_str(&file_contents).unwrap()
    };
}

#[derive(Deserialize, Debug)]
#[serde(crate = "juniper::serde")]
struct SerQuote {
    quote: String,
    author: String,
}

#[derive(Debug)]
pub struct Quote {
    pub text: String,
    pub author: Option<String>,
    pub length: usize,
}

impl Quote {
    pub fn new(text: String, author: Option<String>) -> Self {
        let length = text.len();
        Self {
            text,
            author,
            length,
        }
    }
}

pub fn fetch_quote(length: Length) -> Quote {
    let len = match length {
        Length::Short => 60,
        Length::Medium => 90,
        Length::Long => 120,
    };
    let right_length: Vec<_> = QUOTES
        .iter()
        .filter(|quote| len <= quote.quote.len() && quote.quote.len() < len + 30)
        .collect();

    log::debug!(
        "Number of {:?} length quotes: {:?}",
        length,
        right_length.len()
    );

    let quote = right_length.choose(&mut thread_rng()).unwrap();

    log::trace!("Selected {:?}", quote.quote);

    Quote::new(quote.quote.clone(), Some(quote.author.clone()))
}
