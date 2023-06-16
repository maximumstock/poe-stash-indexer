#!/bin/sh

set -ex

if [[ -v $DATABASE_URL ]]; then
  diesel setup \
    --migration-dir crates/indexer/migrations \
    --config-file crates/indexer/diesel.toml
fi

ls -lah /app
cargo build -vvv --bin indexer
exec cargo watch -w crates -x "run --bin indexer"

