//! This module defines the public interface for the API.

#[macro_use]
extern crate juniper;

use std::env;
use std::sync::{Arc, RwLock};

use iron::prelude::*;
use juniper::{DefaultScalarValue, EmptySubscription, FieldError, FieldResult, Value};
use juniper_iron::{GraphQLHandler, GraphiQLHandler};
use logger::Logger;
use mount::Mount;
use persistent::State;
use postgres::{Client, NoTls};

pub mod ciphers;
pub mod cryptogram;
mod quotes;

pub(crate) use cryptogram::{Answer, Cryptogram, Length, Type};

struct Context {
    db: Client,
}

impl Context {
    pub fn new(client: Client) -> Self {
        Self { db: client }
    }
}

impl iron::typemap::Key for Context {
    type Value = Self;
}

// this is kind of a hack, but it's safe because State gives us a RwLock
// and will prevent deadlocks
unsafe impl Sync for Context {}

type ContextLock = Arc<RwLock<Context>>;

fn context_factory<'a>(r: &'a mut Request) -> IronResult<ContextLock> {
    Ok(r.extensions.get::<State<Context>>().unwrap().clone())
}

struct Query;

#[graphql_object(Context=ContextLock)]
impl Query {
    /// The api version.
    fn api_version() -> &str {
        "0.1"
    }

    /// Request plaintext and key for a specific cryptogram by token.
    fn answer(context: &ContextLock, token: i32) -> FieldResult<Answer> {
        let row = context.write().unwrap().db.query_one(
            "SELECT token, plaintext, key FROM cryptograms WHERE token = $1",
            &[&token],
        );

        match row {
            Ok(r) => {
                let plaintext: String = r.get(1);
                let key: Option<String> = r.get(2);
                println!("plaintext={plaintext:?}, key={key:?}");
                Ok(Answer::new(plaintext, key))
            }
            Err(_) => Err(FieldError::new("Invalid token", Value::null())),
        }
    }
}

struct Mutation;

#[graphql_object(Context=ContextLock)]
impl Mutation {
    /// Request a new ciphertext.
    ///
    /// The argument `key` does nothing if the chosen `Type` does not need a key.
    fn cryptogram(
        context: &ContextLock,
        plaintext: Option<String>,
        length: Option<Length>,
        r#type: Option<Type>,
        key: Option<String>,
    ) -> FieldResult<Cryptogram> {
        let cryptogram = Cryptogram::new(plaintext, length, r#type, key).map_err(|e| {
            FieldError::new(
                "Error constructing cryptogram",
                graphql_value!(format!("{e}")),
            )
        })?;

        println!(
            "inserting token={:?}, plaintext={:?}, key={:?}",
            cryptogram.token, cryptogram.plaintext, cryptogram.key
        );

        context
            .write()
            .unwrap()
            .db
            .execute(
                "INSERT INTO cryptograms (token, plaintext, key) VALUES($1, $2, $3)",
                &[&cryptogram.token, &cryptogram.plaintext, &cryptogram.key],
            )
            .unwrap();

        Ok(cryptogram)
    }
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
        EmptySubscription::<ContextLock>::new(),
    );

    let graphiql_endpoint = GraphiQLHandler::new("/graphql", None);
    mount.mount("/", graphiql_endpoint);
    mount.mount("/graphql", graphql_endpoint);

    let mut chain = Chain::new(mount);
    chain.link(Logger::new(None));

    let pg_url = env::var("CRYPTOGRAMS_PG_URL").expect("Environment variable CRYPTOGRAMS_PG_URL must be set");

    let mut client = Client::connect(&pg_url, NoTls)
        .expect("Could not connect to db");

    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS cryptograms (
            token INT PRIMARY KEY,
            plaintext VARCHAR(160),
            key VARCHAR(20)
        )",
        )
        .unwrap();

    let state = State::<Context>::one(Context::new(client));
    chain.link_before(state);

    let host = env::var("LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".into());

    log::info!("Starting server on {}.", host);
    Iron::new(chain).http(host.as_str()).unwrap();
}
