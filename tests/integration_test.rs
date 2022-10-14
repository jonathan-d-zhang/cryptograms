/// Integration test
///
/// This implements an extremely scuffed custom test harness in order to have setup and teardown
/// functionality. It tries to emulate the output of libtest, but can't really handle anything
/// other than `cargo test`.
#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::panic;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::TempPath;

use regex::Regex;

const URL: &str = "http://localhost:8080/graphql";
const TEST_QUOTE: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";

type StdoutMap = Arc<Mutex<HashMap<String, String>>>;

lazy_static! {
    static ref CLIENT: Client = Client::new();
    static ref STDOUT: StdoutMap = Arc::new(Mutex::new(HashMap::new()));
}

fn main() -> ExitCode {
    let file_handle = setup();

    let tests: &[(fn(), &str)] = &[
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

    println!("\nrunning {} tests", tests.len());

    let mut errors = Vec::with_capacity(tests.len());
    for (test, name) in tests.into_iter() {
        let res = run_test(*test);
        let output = match res {
            Ok(_) => "\x1b[32mok\x1b[37m",
            Err(_) => {
                errors.push(name);
                "\x1b[31mFAILED\x1b[37m"
            }
        };
        println!("test {name} ... {output}");
    }

    let _ = panic::take_hook();

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
        "\ntest result: {}. {} passed; {} failed",
        test_result,
        tests.len() - errors.len(),
        errors.len(),
    );

    std::mem::drop(file_handle);

    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

struct Stdout {
    name: String,
    buf: Vec<u8>,
    map: StdoutMap,
}

impl Stdout {
    pub fn new(name: String, map: StdoutMap) -> Self {
        Self {
            name,
            buf: Vec::new(),
            map,
        }
    }
}

impl Drop for Stdout {
    fn drop(&mut self) {
        self.map.lock().unwrap().insert(
            self.name.clone(),
            String::from_utf8(self.buf.clone()).unwrap(),
        );
    }
}

fn run_test(test: fn()) -> Result<(), Box<dyn std::any::Any + Send>> {
    panic::catch_unwind(test)
}

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
    thread::sleep(std::time::Duration::from_secs(3));

    // set a hook that logs the data instead of printing immediately
    panic::set_hook(Box::new(|info| {
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
        //        println!("{:?}", parts);
        let pat = Regex::new(r"\$test_name=(.+)\$").unwrap();
        let captures = pat.captures(message);
        match captures {
            Some(c) => {
                let test_name = c.get(1).expect("Couldn't find match");

                let mut map = STDOUT.lock().unwrap();
                let entry = map
                    .entry(test_name.as_str().into())
                    .or_insert(String::new());
                entry.push_str(&format!("\n{}", parts.join("\n")));
            }
            None => {
                eprintln!("{}\n{}", message, location_data);
            }
        }
    }));

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

    stdout
        .buf
        .extend_from_slice(format!("{:?}", data).as_bytes());

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

    stdout
        .buf
        .extend_from_slice(format!("{:?}", data).as_bytes());

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
    stdout
        .buf
        .extend_from_slice(format!("{:?}", data).as_bytes());

    assert_eq!(
        data.cryptogram.ciphertext,
        TEST_QUOTE.to_string(),
        "$test_name={}$",
        test_name
    )
}
