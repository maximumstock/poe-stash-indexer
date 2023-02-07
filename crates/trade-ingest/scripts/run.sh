#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-ingest
    ;;
  *)
    exec cargo watch -w crates/trade-ingest/src -x "run --bin trade-ingest"
    ;;
esac

