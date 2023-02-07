#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-api
    ;;
  *)
    exec cargo watch -w crates/trade-api/src -x "run --bin trade-api"
    ;;
esac

