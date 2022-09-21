use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

use std::process::{Command, Child};
use std::path::PathBuf;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
)]
pub struct CipherQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
    response_derives = "Debug"
)]
pub struct VersionQuery;

fn setup_server() -> Child {
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/cryptograms");

    let handle = Command::new(path).spawn().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    handle
}

#[test]
fn test_version() {
    let mut handle = setup_server();
    let client = Client::new();
    let response_body = post_graphql::<VersionQuery, _>(&client, "http://localhost:8080/graphql", version_query::Variables).unwrap();

    let data = response_body.data.unwrap();

    handle.kill().unwrap();
    assert_eq!(data.api_version, "0.1");
}

