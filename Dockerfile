FROM rustlang/rust:nightly-bullseye as test

RUN USER=root cargo new --bin cryptograms
WORKDIR /cryptograms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build

RUN rm target/debug/deps/cryptograms*

COPY tests tests
COPY src src

RUN cargo test --no-run

#########################
FROM rust:1.66-slim-bullseye as build-prod

RUN USER=root cargo new --bin cryptograms
WORKDIR /cryptograms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release

COPY src src
RUN rm target/release/deps/cryptograms*

RUN cargo build --release

#########################
FROM debian:bullseye-slim as prod

COPY --from=build-prod /cryptograms/target/release/cryptograms ./cryptograms
COPY quotes.json quotes.json
COPY words.txt words.txt

CMD ["./cryptograms"]

EXPOSE 8080
