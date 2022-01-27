#!/usr/bin/env bash

set -ex

case $ENV in
  "production")
    exec trade
    ;;
  *)
    exec cargo watch -w trade/src -x "run --bin trade"
    ;;
esac

