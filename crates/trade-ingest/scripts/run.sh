#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-ingest
    ;;
  *)
    exec cargo watch -w crates -x "run --bin trade-ingest"
    ;;
esac

