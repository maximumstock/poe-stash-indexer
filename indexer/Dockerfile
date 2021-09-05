# Based on https://hub.docker.com/_/rust
FROM rust:1.54-slim-buster as base
RUN apt-get update
RUN apt-get install libpq-dev libssl-dev pkg-config -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN cargo install cargo-chef


FROM base as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo build --bin indexer

FROM builder as devserver
WORKDIR /app
CMD ["indexer/scripts/run.sh"]
