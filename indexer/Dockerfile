# Based on https://hub.docker.com/_/rust
FROM rust:1.53-slim-buster
RUN apt-get update
RUN apt-get install libpq-dev -y

RUN cargo install diesel_cli --no-default-features --features postgres
WORKDIR /home/poe-stash-indexer

# Build dependencies
COPY indexer indexer
COPY river-subscription river-subscription
#RUN mkdir -p indexer/src
#RUN echo "fn main() {}" > indexer/src/main.rs
#COPY indexer/Cargo.toml indexer/Cargo.toml
#COPY indexer/Cargo.lock indexer/Cargo.lock
# COPY Cargo.toml Cargo.toml
# COPY Cargo.lock Cargo.lock
# COPY stash-differ stash-differ

#RUN cd indexer && cargo build --release

# Build application
WORKDIR /home/poe-stash-indexer/indexer
RUN cargo build

RUN cp target/debug/indexer bin
