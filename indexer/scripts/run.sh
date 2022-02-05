#!/bin/sh

set -ex

if [[ -v DATABASE_URL ]]; then
  diesel setup \
    --migration-dir indexer/migrations \
    --config-file indexer/diesel.toml
fi

case $ENV in
  "production")
    exec indexer
    ;;
  *)
    exec cargo watch -w indexer/src -x "run --bin indexer"
    ;;
esac

