FROM clux/muslrust:1.60.0 as base
USER root
RUN apt-get update && apt-get upgrade -y
RUN apt-get install libpq-dev libssl-dev pkg-config clang wget -y
# RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
#RUN cargo install cargo-chef

# Install mold
RUN wget https://github.com/rui314/mold/releases/download/v1.2.0/mold-1.2.0-x86_64-linux.tar.gz
RUN tar -xf mold-1.2.0-x86_64-linux.tar.gz
RUN mv mold-1.2.0-x86_64-linux/bin/mold /bin/mold
ENV MOLD_PATH="/bin/mold"
ENV RUSTFLAGS="-Clink-arg=-fuse-ld=$MOLD_PATH -Clinker=clang"
