#![feature(internal_output_capture)]

/// Integration test
///
/// This implements an extremely scuffed custom test harness in order to have setup and teardown
/// functionality. It tries to emulate the output of libtest, but can't handle any flags for `cargo
/// test`.

#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};

use reqwest::blocking::Client;
use std::io::set_output_capture;
use std::panic::catch_unwind;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::TempPath;
const URL: &str = "http://localhost:8080/graphql";


const TEST_QUOTE_SHORT: &str = "The quick brown fox jumps over the lazy dog and the farmer's 7th chicken";
const TEST_QUOTE_MEDIUM: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";
const TEST_QUOTES_JSON: &str = r#"[{"quote": "The quick brown fox jumps over the lazy dog and the farmer's 7th chicken", "author": "jz9", "genre": "testing"},{"quote": "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!", "author": "jz9", "genre": "testing"}]"#;

lazy_static! {
    /// Client for making requests to the API
    static ref CLIENT: Client = Client::new();
}

/// Stores tuples of test and test name
static TESTS: &[(fn(), &str)] = &[
    (test_api_version, "test_api_version"),
    (
        test_medium_identity,
        "test_medium_identity",
    ),
    (test_short_hill_valid_key, "test_short_hill_valid_key"),
    (test_short_hill_invalid_key, "test_short_hill_invalid_key"),
    //    (test_cryptogram_cryptarithm, "test_cryptogram_cryptarithm"),
];

fn main() -> ExitCode {
    let (_file_handles, server_stdout) = setup();
    println!("\nrunning {} tests", TESTS.len());

    // Run the tests
    let mut errors = Vec::with_capacity(TESTS.len());
    for &(test, name) in TESTS.iter() {
        let res = run_test(test);
        let output = match res {
            Ok(_) => "\x1b[32mok\x1b[37m",
            Err(stdout) => {
                errors.push((name, String::from_utf8(stdout).unwrap()));
                "\x1b[31mFAILED\x1b[37m"
            }
        };

        println!("test {name} ... {output}");
    }

    // Print the stdouts of the failed tests
    if !errors.is_empty() {
        println!("\nfailures:");
        for (name, stdout) in &errors {
            let spacer = "-".repeat(4);
            println!("{spacer} {name} stdout {spacer}");
            println!("{stdout}");
        }

        println!("\n\nfailures:");
        for (name, _) in &errors {
            println!("    {name}");
        }
    }

    let test_result = if errors.is_empty() {
        "\x1b[32mok\x1b[37m"
    } else {
        "\x1b[31mFAILED\x1b[37m"
    };

    if !errors.is_empty() {
        println!(
            "{}",
            String::from_utf8(server_stdout.lock().unwrap().clone()).unwrap()
        );
    }

    println!(
        "\ntest result: {}. {} passed; {} failed\n",
        test_result,
        TESTS.len() - errors.len(),
        errors.len(),
    );

    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs a test
///
/// Returns Ok(()) if the test runs without errors. Otherwise, returns the stdout of the test.
fn run_test(f: fn()) -> Result<(), Vec<u8>> {
    let stdout = Arc::new(Mutex::new(Vec::new()));
    set_output_capture(Some(Arc::clone(&stdout)));

    let res = catch_unwind(f);

    // Switch back to stdout
    set_output_capture(None);

    res.map_err(|_| stdout.lock().unwrap().clone())
}

/// Setup tests
///
/// This function does a few setup tasks.
/// - Loads a tempfile for test quotes.
/// - Start the server
fn setup() -> (Vec<TempPath>, Arc<Mutex<Vec<u8>>>) {
    let _ = env_logger::builder()
        .is_test(true)
        .target(env_logger::Target::Stdout)
        .try_init();

    // setup tempfile for quotes
    let quotes_file = tempfile::Builder::new()
        .prefix("test-quotes")
        .suffix(".json")
        .append(true)
        .tempfile_in(std::env::temp_dir())
        .unwrap();

    let quotes_path = quotes_file.path();

    log::debug!("Created temp quotes file at {:?}", quotes_path);

    std::fs::write(quotes_path, TEST_QUOTES_JSON.as_bytes()).unwrap();

    std::env::set_var("QUOTES_FILE", quotes_path);

    let words_file = tempfile::Builder::new()
        .prefix("test-words")
        .suffix(".txt")
        .append(true)
        .tempfile_in(std::env::temp_dir())
        .unwrap();

    let words_path = words_file.path();

    //log::debug!("Created temp words file at {:?}", words_path);

    std::fs::write(words_path, r"send,money,more".as_bytes()).unwrap();

    std::env::set_var("WORDS_FILE", words_path);
    //    std::env::set_var("WORDS_FILE", "words.txt");

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let clone = Arc::clone(&stdout);
    thread::spawn(move || {
        set_output_capture(Some(clone));

        cryptograms::make_server();
    });

    // give some time for the server to start up
    thread::sleep(std::time::Duration::from_secs(3));

    // return temp paths so our temp files stay alive
    (
        vec![quotes_file.into_temp_path(), words_file.into_temp_path()],
        stdout,
    )
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema.graphql",
    query_path = "tests/query.graphql",
    response_derives = "Debug"
)]
pub struct Version;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema.graphql",
    query_path = "tests/query.graphql",
    response_derives = "Debug"
)]
pub struct Answer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema.graphql",
    query_path = "tests/query.graphql",
    response_derives = "Debug"
)]
pub struct Cryptogram;

fn test_api_version() {
    let response_body = post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();
    println!("{data:?}");

    assert_eq!(data.api_version, "0.1")
}

// First tests that the api returns the correct response
// Then checks that it returns the correct plaintext when requested with the token it gives
fn test_medium_identity() {
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::IDENTITY),
        length: Some(cryptogram::Length::MEDIUM),
        key: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();

    println!("{data:?}");

    assert_eq!(
        data.cryptogram.ciphertext,
        "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG. CAN'T-I'M<>12932. CWM FJORD BANK GLYPHS VEXT QUIZ!",
    );

    let variables = answer::Variables {
        token: data.cryptogram.token
    };

    let response_body = post_graphql::<Answer, _>(&CLIENT, URL, variables).unwrap();

    let data: answer::ResponseData = response_body.data.unwrap();
    println!("{data:?}");

    assert_eq!(
        data.answer.plaintext,
        TEST_QUOTE_MEDIUM,
    );
}

fn test_short_hill_valid_key() {
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::HILL),
        length: Some(cryptogram::Length::SHORT),
        key: Some(String::from("abcd")),
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();

    println!("{data:?}");

    assert_eq!(
        data.cryptogram.ciphertext,
        "HHQEIMKIRBWQFPXTUAPROAECTNEAAWYSOWAMDJHHFXRZEKSKHHHZCWEGZX",
    );

    let variables = answer::Variables {
        token: data.cryptogram.token
    };

    let response_body = post_graphql::<Answer, _>(&CLIENT, URL, variables).unwrap();

    assert!(response_body.errors.is_none());

    let data: answer::ResponseData = response_body.data.unwrap();
    println!("{data:?}");

    assert_eq!(
        data.answer.plaintext,
        TEST_QUOTE_SHORT,
    );
}

fn test_short_hill_invalid_key() {
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::HILL),
        length: Some(cryptogram::Length::SHORT),
        key: Some(String::from("aaa")),
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables);

    println!("{response_body:?}");

    let error = response_body.unwrap().errors.unwrap();

    assert_eq!(error[0].message, "KeyError: Key length must be a perfect square")
}

/*
fn test_cryptogram_cryptarithm() {
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::CRYPTARITHM),
        length: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    // any permutation of the 3 words is possible
    let words: HashSet<&str> = data
        .cryptogram
        .ciphertext
        .split_whitespace()
        .filter(|&w| w != "+" && w != "=")
        .collect();

    assert_eq!(words, HashSet::from(["more", "money", "send"]));
    assert!(false);
}
*/
