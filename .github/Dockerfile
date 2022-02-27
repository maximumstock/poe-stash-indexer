FROM ekidd/rust-musl-builder:1.57.0 as base
USER root
RUN apt-get update
RUN apt-get install libpq-dev libssl-dev pkg-config lld -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN cargo install cargo-chef

ENV RUSTFLAGS="-Clink-arg=-fuse-ld=lld" 