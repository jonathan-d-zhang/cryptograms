#![feature(internal_output_capture)]

/// Integration test
///
/// This implements an extremely scuffed custom test harness in order to have setup and teardown
/// functionality. It tries to emulate the output of libtest, but can't handle any flags for `cargo
/// test`.
///
///
/// # How to write tests for this module
/// - Tests in this file are not annotated with `#[test]`. Instead, they must be registered in [`TESTS`].
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

use env_logger;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use log;
use reqwest::blocking::Client;
use std::panic::catch_unwind;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::set_output_capture;
use tempfile::TempPath;

const URL: &str = "http://localhost:8080/graphql";
const TEST_QUOTE: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";

lazy_static! {
    /// Client for making requests to the API
    static ref CLIENT: Client = Client::new();
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
    let (_file_handle, server_stdout) = setup();
    println!("\nrunning {} tests", TESTS.len());

    // Run the tests
    let mut errors = Vec::with_capacity(TESTS.len());
    for &(test, name) in TESTS.into_iter() {
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

    println!(
        "\ntest result: {}. {} passed; {} failed\n",
        test_result,
        TESTS.len() - errors.len(),
        errors.len(),
    );

    println!("{}", String::from_utf8(server_stdout.lock().unwrap().clone()).unwrap());

    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs a test
///
/// Returns Ok(()) if the test runs without errors. Otherwise, returns the stdout of the test.
///
/// The capturing is thread-local, so the stdout of the server itself can't be captured.
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
fn setup() -> (TempPath, Arc<Mutex<Vec<u8>>>) {
    let _ = env_logger::builder()
        .is_test(true)
        .target(env_logger::Target::Stdout)
        .try_init();

    // setup tempfile for quotes
    let tf = tempfile::Builder::new()
        .prefix("test-quotes")
        .suffix(".json")
        .append(true)
        .tempfile_in(std::env::temp_dir())
        .unwrap();

    let path = tf.path();

    log::debug!("Created temp quotes file at {:?}", path);

    std::fs::write(&path,
        r#"[{"quote": "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!", "author": "jz9", "genre": "testing"}]"#
        .as_bytes(),
    )
    .unwrap();

    std::env::set_var("QUOTES_FILE", path);

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let clone = Arc::clone(&stdout);
    thread::spawn(move || {
        set_output_capture(Some(clone));

        cryptograms::make_server();
    });

    // give some time for the server to start up
    thread::sleep(std::time::Duration::from_secs(3));

    // return a temp path so our temp file stays alive
    (tf.into_temp_path(), stdout)
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

    let response_body = post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    assert_eq!(data.api_version, "0.1", "$test_name={}$", test_name)
}

fn test_cryptogram_identity_medium() {
    let test_name = "test_cryptogram_identity_medium";

    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::IDENTITY),
        length: Some(cryptogram::Length::MEDIUM),
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    assert_eq!(
        data.cryptogram.ciphertext,
        "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!",
        "$test_name={}$", test_name
    )
}

fn test_cryptogram_encrypt_identity() {
    let test_name = "test_cryptogram_encrypt_identity";
    let variables = cryptogram::Variables {
        plaintext: Some(TEST_QUOTE.to_string()),
        type_: None,
        length: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    assert_eq!(
        data.cryptogram.ciphertext,
        TEST_QUOTE.to_string(),
        "$test_name={}$",
        test_name
    )
}
