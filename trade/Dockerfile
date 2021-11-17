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

FROM base as development-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .

FROM development-builder-deps as development-builder
WORKDIR /app
COPY . .
RUN cargo build --bin trade

FROM development-builder as development
WORKDIR /app
COPY . .
ENTRYPOINT ["trade/scripts/run.sh"]

FROM base as production-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

FROM production-builder-deps as production-builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin trade

FROM base as production
WORKDIR /app
COPY . .
COPY --from=production-builder /app/target/release/trade /bin/trade
ENTRYPOINT ["trade/scripts/run.sh"]
