#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-ingest
    ;;
  *)
    exec cargo watch -w trade-ingest/src -x "run --bin trade-ingest --release"
    ;;
esac

