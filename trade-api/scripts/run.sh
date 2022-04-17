#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-api
    ;;
  *)
    exec cargo watch -w trade-api/src -x "run --bin trade-api --release"
    ;;
esac

