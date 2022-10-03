


#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use std::sync::Once;
use std::thread;

const URL: &str = "http://localhost:8080/graphql";
const TEST_QUOTE: &str = "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!";
static SETUP: Once = Once::new();

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

fn setup() {
    SETUP.call_once(|| {
        thread::spawn(|| {
            // setup tempfile for quotes
            println!("Temp dir {:?}", std::env::temp_dir());
            let tf = tempfile::Builder::new()
                .prefix("test-quotes")
                .suffix(".json")
                .append(true)
                .tempfile_in(std::env::temp_dir())
                .unwrap();

            let path = tf.path();
            println!("Wrote to {:?}", path);

            std::fs::write(&path,
                r#"[{"quote": "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!", "author": "jz9", "genre": "testing"}]"#
                .as_bytes(),
            )
            .unwrap();

            std::env::set_var(
                "QUOTES_FILE",
                path,
            );
            cryptograms::make_server();
        });
        thread::sleep(std::time::Duration::from_secs(3));
    });
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

#[test]
fn test_api_version() {
    setup();

    let response_body = post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    assert_eq!(data.api_version, "0.1")
}

#[test]
fn test_cryptogram_identity_medium() {
    setup();

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
        "The quick brown fox jumps over the lazy dog. Can't-I'm<>12932. Cwm fjord bank glyphs vext quiz!"
    )
}

#[test]
fn test_cryptogram_encrypt_identity() {
    setup();

    let variables = cryptogram::Variables {
        plaintext: Some(TEST_QUOTE.to_string()),
        type_: None,
        length: None,
    };

    let response_body = post_graphql::<Cryptogram, _>(&CLIENT, URL, variables).unwrap();

    let data: cryptogram::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);

    assert_eq!(data.cryptogram.ciphertext, TEST_QUOTE.to_string(),)
}
