FROM rust:1.62 as base

RUN USER=root cargo new --bin cryptograms
WORKDIR /cryptograms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build

COPY src src
RUN rm target/debug/deps/cryptograms*

RUN cargo build

#########################
FROM debian:bullseye-slim as dev

COPY --from=base /cryptograms/target/debug/cryptograms ./cryptograms
COPY quotes.json quotes.json

CMD ["./cryptograms"]

EXPOSE 8080
