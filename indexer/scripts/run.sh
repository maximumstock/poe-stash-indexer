#!/usr/bin/env bash

set -ex

while !</dev/tcp/db/5432; do sleep 1; done;

diesel setup \
  --migration-dir indexer/migrations \
  --config-file indexer/diesel.toml

case $ENV in
  "production")
    indexer
    ;;
  *)
    cargo watch -x "run --bin indexer"
    ;;
esac

