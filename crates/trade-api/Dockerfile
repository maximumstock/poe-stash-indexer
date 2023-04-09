FROM rust:bullseye as development
# Install & integrate mold for faster dev link times
# Add apt.llvm.org repository and install clang
RUN apt-get update
RUN apt-get install -y wget software-properties-common
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-add-repository "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch main"
RUN apt-get install -y clang
RUN apt-get install -y protobuf-compiler
RUN wget https://github.com/rui314/mold/releases/download/v1.11.0/mold-1.11.0-aarch64-linux.tar.gz
RUN tar -xf mold-1.11.0-aarch64-linux.tar.gz
RUN mv mold-1.11.0-aarch64-linux/bin/mold /bin/mold
ENV MOLD_PATH="/bin/mold"
ENV RUSTFLAGS="-Clink-arg=-fuse-ld=$MOLD_PATH -Clinker=clang"
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true CARGO_UNSTABLE_SPARSE_REGISTRY=true cargo install cargo-watch
WORKDIR /app
COPY . .
ENTRYPOINT ["bash", "crates/trade-api/scripts/run.sh"]

# FROM maximumstock2/base:latest as planner
# WORKDIR /app
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM planner as production-builder-deps
# WORKDIR /app
# COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json --package trade-api

FROM maximumstock2/base:latest as production-builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin trade-api
RUN strip /app/target/x86_64-unknown-linux-musl/release/trade-api

FROM alpine as production
WORKDIR /app
USER root
COPY --from=production-builder /app/target/x86_64-unknown-linux-musl/release/trade-api /usr/local/bin/
CMD ["/usr/local/bin/trade-api"]
