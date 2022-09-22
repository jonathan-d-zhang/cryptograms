#[macro_use]
extern crate juniper;
use std::env;

use iron::prelude::*;
use juniper::{DefaultScalarValue, EmptyMutation, EmptySubscription};
use juniper_iron::{GraphQLHandler, GraphiQLHandler};
use logger::Logger;
use mount::Mount;

mod ciphers;
use ciphers::Type;
struct Query;

#[graphql_object]
impl Query {
    /// The api version.
    fn api_version() -> &str {
        "0.1"
    }

    /// Request a new ciphertext.
    fn cipher(r#type: ciphers::Type) -> ciphers::Cipher {
        let plaintext = "a";

        match r#type {
            Type::Rot13 => ciphers::Cipher::new(plaintext),
        }
    }
}

fn context_factory(_: &mut Request) -> IronResult<()> {
    Ok(())
}

pub fn make_server() {
    let mut mount = Mount::new();

    let graphql_endpoint = <GraphQLHandler<_, _, _, _, _, DefaultScalarValue>>::new(
        context_factory,
        Query,
        EmptyMutation::<()>::new(),
        EmptySubscription::<()>::new(),
    );

    let graphiql_endpoint = GraphiQLHandler::new("/graphql", None);
    mount.mount("/", graphiql_endpoint);
    mount.mount("/graphql", graphql_endpoint);

    let (logger_before, logger_after) = Logger::new(None);

    let mut chain = Chain::new(mount);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    let host = env::var("LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".into());
    Iron::new(chain).http(host.as_str()).unwrap();
}

