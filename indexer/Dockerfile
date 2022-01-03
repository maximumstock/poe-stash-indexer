FROM rust:1.54-slim-buster as base
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

FROM base as development-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json -p indexer

FROM development-builder-deps as development-builder
WORKDIR /app
COPY . .
RUN cargo build --bin indexer

FROM development-builder as development
WORKDIR /app
COPY . .
ENTRYPOINT ["indexer/scripts/run.sh"]

FROM base as production-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p indexer

FROM production-builder-deps as production-builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin indexer
RUN strip /app/target/release/indexer

FROM scratch as production
WORKDIR /app
COPY . .
COPY --from=production-builder /app/target/release/indexer /bin/indexer
ENTRYPOINT ["indexer/scripts/run.sh"]
