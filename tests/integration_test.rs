/// Integration test
///
/// This implements an extremely scuffed custom test harness in order to have setup and teardown
/// functionality. It tries to emulate the output of libtest, but can't really handle anything
/// other than `cargo test`.
#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use std::thread;
use std::panic;
use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::Mutex;

const URL: &str = "http://localhost:8080/graphql";
const TEST_QUOTE: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";

lazy_static!{
    static ref CLIENT: Client = Client::new();
    static ref STDOUT: Mutex<HashMap<&'static str, Vec<u8>>> = Mutex::new(HashMap::new());
}

fn main() -> ExitCode {
    setup();

    let tests: &[(fn(), &str)] = &[
        (test_api_version, "test_api_version"),
        (test_cryptogram_identity_medium, "test_cryptogram_identity_medium"),
        (test_cryptogram_encrypt_identity, "test_cryptogram_encrypt_identity"),
    ];

    println!("running {} tests", tests.len());

    let mut errors = Vec::with_capacity(tests.len());
    for (test, name) in tests.into_iter() {
        let res = run_test(*test);
        let output = match res {
            Ok(_) => "\x1b[32mok\x1b[37m",
            Err(e) => {
                errors.push((name, e));
                "\x1b[31mFAILED\x1b[37m"
            }
        };
        println!("test {name} ... {output}");
    }

    let _ = panic::take_hook();

    if !errors.is_empty() {
        println!("\nfailures:");
        for (&name, error) in &errors {
            let spacer = "-".repeat(4);
            println!("{spacer} {name} stdout {spacer}");
            print!("{}", String::from_utf8(STDOUT.lock().unwrap()[name].clone()).unwrap());

            if let Some(p) = error.downcast_ref::<String>() {
                println!("{p:?}");
            } else if let Some(p) = error.downcast_ref::<&str>() {
                println!("{p:?}");
            }
        }

        println!("\nfailures:");
        for (name, _) in &errors {
            println!("\t{name}")
        }
    }

    println!("\ntest result {} failed; {} passed", errors.len(), tests.len() - errors.len());

    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

struct Stdout {
    name: &'static str,
    buf: Vec<u8>,
}

impl Stdout {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            buf: Vec::new(),
        }
    }
}

impl Drop for Stdout {
    fn drop(&mut self) {
        STDOUT.lock().unwrap().insert(self.name, self.buf.clone());
    }
}

fn run_test(test: fn()) -> Result<(), Box<dyn std::any::Any + Send>> {
    panic::catch_unwind(test)
}

fn setup() {
    // setup tempfile for quotes
    let tf = tempfile::Builder::new()
        .prefix("test-quotes")
        .suffix(".json")
        .append(true)
        .tempfile_in(std::env::temp_dir())
        .unwrap();

    let path = tf.path();

    std::fs::write(&path,
        r#"[{"quote": "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!", "author": "jz9", "genre": "testing"}]"#
        .as_bytes(),
    )
    .unwrap();

    std::env::set_var(
        "QUOTES_FILE",
        path,
    );
    thread::spawn(|| {
        cryptograms::make_server();
    });
    thread::sleep(std::time::Duration::from_secs(3));

    // set a hook that doesn't print as soon as a panic occurs
    panic::set_hook(Box::new(|_| {}));
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
    let mut stdout = Stdout::new("test_api_version");

    let response_body = post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();

    stdout.buf.extend_from_slice(format!("{:?}", data).as_bytes());

    assert_eq!(data.api_version, "0.1")
}

fn test_cryptogram_identity_medium() {
    let mut stdout = Stdout::new("test_cryptogram_identity_medium");
    let variables = cryptogram::Variables {
        plaintext: None,
        type_: Some(cryptogram::Type::IDENTITY),
        length: Some(cryptogram::Length::MEDIUM),
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();

    stdout.buf.extend_from_slice(format!("{:?}", data).as_bytes());

    assert_eq!(
        data.cryptogram.ciphertext,
        "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!"
    )
}

fn test_cryptogram_encrypt_identity() {
    let mut stdout = Stdout::new("test_cryptogram_encrypt_identity");
    let variables = cryptogram::Variables {
        plaintext: Some(TEST_QUOTE.to_string()),
        type_: None,
        length: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();
    stdout.buf.extend_from_slice(format!("{:?}", data).as_bytes());

    assert_eq!(data.cryptogram.ciphertext, TEST_QUOTE.to_string(),)
}
