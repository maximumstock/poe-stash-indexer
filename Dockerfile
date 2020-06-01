# https://blog.sedrik.se/posts/my-docker-setup-for-rust/
FROM ekidd/rust-musl-builder as builder

WORKDIR /home/rust/

# Avoid having to install/build all dependencies by copying
# the Cargo files and making a dummy src/main.rs
COPY Cargo.toml .
COPY Cargo.lock .
RUN echo "fn main() {}" > src/main.rs
RUN cargo test
RUN cargo build
# --release

# We need to touch our real main.rs file or else docker will use
# the cached one.
COPY . .
RUN sudo touch src/main.rs
RUN sudo chown -R rust:rust target/debug/.cargo-lock

RUN cargo test
RUN cargo build
# --release

# Size optimization
# RUN strip target/x86_64-unknown-linux-musl/debug/poe-stash-indexer

# Start building the final image
FROM scratch
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/debug/poe-stash-indexer .
ENTRYPOINT ["./poe-stash-indexer"]
