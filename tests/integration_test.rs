/// Integration test
///
/// This implements an extremely scuffed custom test harness in order to have setup and teardown
/// functionality. It tries to emulate the output of libtest, but can't handle any flags for `cargo
/// test`.
///
/// This module implements a custom panic hook in order to delay the printing of failed assertions.
///
/// # How to write tests for this module
/// - Tests in this file are not annotated with `#[test]`. Instead, they must be registered in [`TESTS`].
/// - To print to stdout, tests use the [`stdout::Stdout`] struct and push lines with the
/// [`stdout::Stdout::push`] method.
/// attribute
/// - Assertions must include the name of the test as a custom message
/// - Tests should include the name of test as a custom message for the assertion. See the example
/// for more detail.
///
/// # Example Test
/// ```
/// fn test() {
///     let test_name = "test";
///     let stdout = Stdout::new(test_name.into(), Arc::clone(STDOUT));
///
///     // each string added will be printed on a new line
///     stdout.push("Logging some data");
///
///     assert_eq!(1 + 1, 2, "$test_case={}$", test_name)
/// }
/// ```
/// The test case name must be provided in this exact format (`$test_case=test_name$`).
#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::panic::{self, PanicInfo};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::TempPath;

use regex::Regex;

mod stdout;
use stdout::Stdout;

const URL: &str = "http://localhost:8080/graphql";
const TEST_QUOTE: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";

lazy_static! {
    /// Client for making requests to the API
    static ref CLIENT: Client = Client::new();
    static ref STDOUT: stdout::StdoutMap = Arc::new(Mutex::new(HashMap::new()));
}

/// Stores tuples of test and test name
static TESTS: &[(fn(), &str)] = &[
    (test_api_version, "test_api_version"),
    (
        test_cryptogram_identity_medium,
        "test_cryptogram_identity_medium",
    ),
    (
        test_cryptogram_encrypt_identity,
        "test_cryptogram_encrypt_identity",
    ),
];

fn main() -> ExitCode {
    let _file_handle = setup();

    println!("\nrunning {} tests", TESTS.len());

    // Run the tests
    let mut errors = Vec::with_capacity(TESTS.len());
    for (test, name) in TESTS.into_iter() {
        let res = panic::catch_unwind(test);
        let output = match res {
            Ok(_) => "\x1b[32mok\x1b[37m",
            Err(_) => {
                errors.push(name);
                "\x1b[31mFAILED\x1b[37m"
            }
        };
        println!("test {name} ... {output}");
    }

    // Unregister our custom hook
    let _ = panic::take_hook();

    // Print the stdouts of the failed tests
    if !errors.is_empty() {
        println!("\nfailures:");
        for &&name in &errors {
            let spacer = "-".repeat(4);
            println!("{spacer} {name} stdout {spacer}");
            print!("{}", STDOUT.lock().unwrap()[name].clone());
        }

        println!("\n\nfailures:");
        for name in &errors {
            println!("    {name}")
        }
    }

    let test_result = if errors.is_empty() {
        "\x1b[32mok\x1b[37m"
    } else {
        "\x1b[31mFAILED\x1b[37m"
    };

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

/// Custom panic hook that logs failed assertions instead of printing them
///
/// If a test assertion panics, the hook logs the message in [`STDOUT`] so it can be displayed with the
/// rest of the stdout for that test case. This does not affect the printing of other panics.
fn hook(info: &PanicInfo) {
    let t = info.payload();
    let location_data = if let Some(d) = info.location() {
        format!("{}:{}:{}", d.file(), d.line(), d.column())
    } else {
        "No location data".into()
    };

    let message = if let Some(p) = t.downcast_ref::<String>() {
        p
    } else if let Some(p) = t.downcast_ref::<&str>() {
        p
    } else {
        ""
    };

    let mut parts = Vec::new();
    parts.push(message);
    parts.push(&location_data);

    let pat = Regex::new(r"\$test_name=(.+)\$").unwrap();
    let captures = pat.captures(message);
    match captures {
        // If the regex matches, then this panic was raised by a test assertion.
        // Append the message onto the existing output for this test.
        Some(c) => {
            let test_name = c.get(1).expect("Couldn't find match");

            let mut map = STDOUT.lock().unwrap();
            let entry = map
                .entry(test_name.as_str().into())
                .or_insert(String::new());
            entry.push_str(&format!("\n{}", parts.join("\n")));
        }
        // Else, it was a normal panic, just print it normally
        None => {
            eprintln!("{}\n{}", message, location_data);
        }
    }
}

/// Setup tests
///
/// This function does a few setup tasks.
/// - Loads a tempfile for test quotes.
/// - Sets our custom panic hook.
/// - Start the server
fn setup() -> TempPath {
    // setup tempfile for quotes
    let tf = tempfile::Builder::new()
        .prefix("test-quotes")
        .suffix(".json")
        .append(true)
        .tempfile_in(std::env::temp_dir())
        .unwrap();

    let path = tf.path();

    println!("Creating temp file at {:?}", path);

    std::fs::write(&path,
        r#"[{"quote": "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!", "author": "jz9", "genre": "testing"}]"#
        .as_bytes(),
    )
    .unwrap();

    std::env::set_var("QUOTES_FILE", path);

    thread::spawn(|| {
        cryptograms::make_server();
    });

    // give some time for server to start up
    thread::sleep(std::time::Duration::from_secs(3));

    // set panics to use our custom hook
    panic::set_hook(Box::new(hook));

    // return a temp path so our temp file stays alive
    tf.into_temp_path()
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
pub struct Cryptogram;

fn test_api_version() {
    let test_name = "test_api_version";
    let mut stdout = Stdout::new(test_name.into(), Arc::clone(&STDOUT));

    let response_body = post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();

    stdout.push(format!("{:?}", data));

    assert_eq!(data.api_version, "0.1", "$test_name={}$", test_name)
}

fn test_cryptogram_identity_medium() {
    let test_name = "test_cryptogram_identity_medium";
    let mut stdout = Stdout::new(test_name.into(), Arc::clone(&STDOUT));
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::IDENTITY),
        length: Some(cryptogram::Length::MEDIUM),
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();

    stdout.push(format!("{:?}", data));

    assert_eq!(
        data.cryptogram.ciphertext,
        "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!",
        "$test_name={}$", test_name
    )
}

fn test_cryptogram_encrypt_identity() {
    let test_name = "test_cryptogram_encrypt_identity";
    let mut stdout = Stdout::new(test_name.into(), Arc::clone(&STDOUT));
    let variables = cryptogram::Variables {
        plaintext: Some(TEST_QUOTE.to_string()),
        type_: None,
        length: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();

    stdout.push(format!("{:?}", data));

    assert_eq!(
        data.cryptogram.ciphertext,
        TEST_QUOTE.to_string(),
        "$test_name={}$",
        test_name
    )
}
