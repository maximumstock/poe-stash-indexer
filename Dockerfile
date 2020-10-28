# Based on https://hub.docker.com/_/rust
FROM rust:latest
RUN cargo install diesel_cli --no-default-features --features postgres
WORKDIR /usr/src/myapp

# Build dependencies
RUN mkdir indexer
RUN mkdir indexer/src
RUN echo "fn main() {}" > indexer/src/main.rs

COPY indexer/Cargo.toml indexer/Cargo.toml
COPY indexer/Cargo.lock indexer/Cargo.lock

COPY lib lib

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release

# Build application
COPY . .
RUN cargo build --release
RUN cargo test --release

RUN cp /usr/src/myapp/target/release/poe-stash-indexer /usr/local/bin/myapp
# CMD ["myapp"]
