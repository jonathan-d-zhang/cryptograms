use cryptograms;

#[macro_use]
extern crate lazy_static;

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use std::sync::Once;
use std::thread;

const URL: &str = "http://localhost:8080/graphql";
static SETUP: Once = Once::new();

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

fn setup() {
    SETUP.call_once(|| {
        thread::spawn(|| cryptograms::make_server());
    });
    thread::sleep(std::time::Duration::from_secs(5));
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
    response_derives = "Debug"
)]
pub struct Version;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
    response_derives = "Debug"
)]
pub struct Cipher;

#[test]
fn test_api_version() {
    setup();

    let response_body =
        post_graphql::<Version, _>(&CLIENT, URL, version::Variables).unwrap();

    let data: version::ResponseData = response_body.data.unwrap();
    println!("{:?}", data);
}

#[test]
fn test_cipher() {
    setup();

    let variables = cipher::Variables {
        type_: cipher::Type::ROT13,
    };

    let response_body = post_graphql::<Cipher, _>(&CLIENT, URL, variables).unwrap();

    println!("{:?}", response_body);

//    let data: cipher::ResponseData = response_body.data.unwrap();
//    println!("{:?}", data);
}

