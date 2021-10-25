#!/usr/bin/env bash

set -ex

while !</dev/tcp/db/5432; do sleep 1; done;

if [[ -v DATABASE_URL ]]; then
  diesel setup \
    --migration-dir indexer/migrations \
    --config-file indexer/diesel.toml
fi

case $ENV in
  "production")
    indexer
    ;;
  *)
    cargo watch -w indexer -x "run --bin indexer"
    ;;
esac

