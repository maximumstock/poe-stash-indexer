FROM ekidd/rust-musl-builder:1.57.0 as base
USER root
RUN apt-get update
RUN apt-get install libpq-dev libssl-dev pkg-config lld -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN cargo install cargo-chef

ENV RUSTFLAGS="-Clink-arg=-fuse-ld=lld" 

FROM base as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base as development
WORKDIR /app
COPY . .
ENTRYPOINT ["trade/scripts/run.sh"]

FROM base as production-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --package trade
COPY . .

FROM production-builder-deps as production-builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin trade
RUN strip /app/target/x86_64-unknown-linux-musl/release/trade 

FROM alpine
WORKDIR /app
COPY --from=production-builder /app/target/x86_64-unknown-linux-musl/release/trade /usr/local/bin/
CMD ["/usr/local/bin/trade"]
