FROM maximumstock2/base:latest as development
WORKDIR /app
COPY . .
ENTRYPOINT ["indexer/scripts/run.sh"]

FROM maximumstock2/base:latest as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM planner as production-builder-deps
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json -p indexer

FROM production-builder-deps as production-builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin indexer
RUN strip /app/target/x86_64-unknown-linux-musl/release/indexer 

FROM alpine
WORKDIR /app
COPY --from=production-builder /app/target/x86_64-unknown-linux-musl/release/indexer /usr/local/bin/
CMD ["/usr/local/bin/indexer"]
