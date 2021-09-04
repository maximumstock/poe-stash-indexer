# Based on https://hub.docker.com/_/rust
FROM rust:1.54-slim-buster as base
RUN apt-get update
RUN apt-get install libpq-dev libssl-dev pkg-config -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

FROM base as builder
WORKDIR /usr/local/src/poe-stash-indexer
# Pre-build stash-api dependencies
COPY stash-api ./stash-api
RUN echo "#[allow(dead_code)]\nfn main() {}" > stash-api/src/lib.rs
RUN cd stash-api && cargo build
COPY stash-api/src ./stash-apis/src

# Pre-build indexer dependencies
COPY indexer ./indexer
RUN echo "#[allow(dead_code)]\nfn main() {}" > indexer/src/main.rs
RUN cd indexer && cargo build
COPY stash-api/src ./stash-apis/src

WORKDIR /usr/local/src/poe-stash-indexer/indexer
RUN cargo build
RUN cp target/debug/indexer /usr/local/bin/indexer

FROM builder as devserver
WORKDIR /usr/local/src/poe-stash-indexer/indexer
CMD ["scripts/run.sh"]