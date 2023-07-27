#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade-api
    ;;
  *)
    exec cargo watch -w crates -x "run --bin trade-api"
    ;;
esac

