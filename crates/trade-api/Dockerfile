# Development Build
FROM rust:bullseye as development
# Install & integrate mold for faster dev link times
# Add apt.llvm.org repository and install clang
RUN apt-get update
RUN apt-get install -y protobuf-compiler
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true CARGO_UNSTABLE_SPARSE_REGISTRY=true cargo install cargo-watch
WORKDIR /app
COPY . .
ENTRYPOINT ["bash", "crates/trade-api/scripts/run.sh"]

# CI/Production Build
FROM alpine as production
COPY ./target/x86_64-unknown-linux-musl/release/trade-api /usr/local/bin/
CMD ["/usr/local/bin/trade-api"]
