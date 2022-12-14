//! This module defines the public interface for the API.

#[macro_use]
extern crate juniper;
use std::env;

use iron::prelude::*;
use juniper::{DefaultScalarValue, EmptySubscription};
use juniper_iron::{GraphQLHandler, GraphiQLHandler};
use logger::Logger;
use mount::Mount;

mod ciphers;
mod cryptogram;
mod quotes;
use cryptogram::{Cryptogram, Length, Type};

struct Query;

#[graphql_object]
impl Query {
    /// The api version.
    fn api_version() -> &str {
        "0.1"
    }

    /// Request plaintext for a specific cryptogram by token.
    fn plaintext(token: String) -> &str {
        "the cipher"
    }
}

struct Mutation;

#[graphql_object]
impl Mutation {
    /// Request a new ciphertext.
    ///
    /// The argument `key` does nothing if the chosen `Type` does not need a key.
    fn cryptogram(
        ciphertext: Option<String>,
        length: Option<Length>,
        r#type: Option<Type>,
        key: Option<String>,
    ) -> Cryptogram {
        Cryptogram::new(ciphertext, length, r#type, key)
    }
}

fn context_factory(_: &mut Request) -> IronResult<()> {
    Ok(())
}

/// Output the current GraphQL schema.
pub fn print_schema() {
    use juniper::RootNode;
    let schema = RootNode::new(Query, Mutation, EmptySubscription::<()>::new());
    println!("{}", schema.as_schema_language());
}

/// Start the GraphQL server.
pub fn make_server() {
    let mut mount = Mount::new();

    let graphql_endpoint = <GraphQLHandler<_, _, _, _, _, DefaultScalarValue>>::new(
        context_factory,
        Query,
        Mutation,
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
