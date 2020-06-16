# Based on https://hub.docker.com/_/rust
FROM rust:latest
RUN cargo install diesel_cli --no-default-features --features postgres
WORKDIR /usr/src/myapp

RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

# Build dependencies
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build

# Build application
COPY . .
RUN cargo build
RUN cargo test

RUN cp /usr/src/myapp/target/debug/poe-stash-indexer /usr/local/bin/myapp
# CMD ["myapp"]
