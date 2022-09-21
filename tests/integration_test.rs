use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

use std::path::PathBuf;
use std::process::{Child, Command};

use std::sync::Once;

static SETUP: Once = Once::new();

static mut SERVER: Server = Server::new();

struct Server {
    handle: Option<Child>,
}

impl Server {
    const fn new() -> Self{
        Server {
            handle: None,
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Safety: this should be ok, since we should only drop `Server` when all the tests are
        // done
        self.handle.as_mut().unwrap().kill().unwrap();
    }
}

fn setup_server() {
    SETUP.call_once(|| {
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/cryptograms");

    unsafe {
        SERVER.handle = Some(Command::new(path).spawn().unwrap());
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    });
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
    response_derives = "Debug"
)]
pub struct VersionQuery;

#[test]
fn test_version() {
    setup_server();

    let client = Client::new();
    let response_body = post_graphql::<VersionQuery, _>(
        &client,
        "http://localhost:8080/graphql",
        version_query::Variables,
    )
    .unwrap();

    let data = response_body.data.unwrap();

    assert_eq!(data.api_version, "0.1");
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/test_schema.graphql",
    query_path = "tests/test_query.graphql",
    response_derives = "Debug"
)]
pub struct CipherQuery;

#[test]
fn test_cipher() {
    
}
