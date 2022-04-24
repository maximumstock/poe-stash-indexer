FROM ekidd/rust-musl-builder:1.57.0 as base
USER root
RUN apt-get update && apt-get upgrade -y
RUN apt-get install libpq-dev libssl-dev pkg-config clang wget -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN cargo install cargo-chef

# Install mold
RUN wget https://github.com/rui314/mold/releases/download/v1.2.0/mold-1.2.0-x86_64-linux.tar.gz
RUN tar -xf mold-1.2.0-x86_64-linux.tar.gz
RUN cp mold-1.2.0-x86_64-linux/bin/mold /usr/bin/mold

ENV RUSTFLAGS="-Clink-arg=-fuse-ld=lld"
ENV LDFLAGS="-fuse-ld=/usr/bin/mold";
ENV RUSTFLAGS = "-Clink-arg=-fuse-ld=/usr/bin/mold -Clinker=clang";
