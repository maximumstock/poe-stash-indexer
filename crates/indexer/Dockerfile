# Development Build
FROM rust:bullseye as development
# Install & integrate mold for faster dev link times
# Add apt.llvm.org repository and install clang
RUN apt-get update
RUN apt-get install -y protobuf-compiler
RUN cargo install cargo-watch
WORKDIR /app
COPY . .
ENTRYPOINT ["bash", "crates/indexer/scripts/run.sh"]

# CI/Production Build
FROM alpine as production
COPY ./target/x86_64-unknown-linux-musl/release/indexer /usr/local/bin/
CMD ["/usr/local/bin/indexer"]
