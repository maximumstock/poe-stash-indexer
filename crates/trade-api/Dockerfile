FROM maximumstock2/base:latest as development
WORKDIR /app
COPY . .
ENTRYPOINT ["crates/trade-api/scripts/run.sh"]

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
