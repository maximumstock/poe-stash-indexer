# Based on https://hub.docker.com/_/rust
FROM rust:1.53-slim-buster as builder
RUN apt-get update
RUN apt-get install libpq-dev -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

WORKDIR /usr/local/src/poe-stash-indexer
COPY indexer ./indexer
COPY river-subscription ./river-subscription

WORKDIR /usr/local/src/poe-stash-indexer/indexer
RUN cargo build
RUN cp target/debug/indexer /usr/local/bin/indexer

FROM builder as devserver
WORKDIR /usr/local/src/poe-stash-indexer/indexer
CMD ["scripts/run.sh"]