#!/usr/bin/env sh

set -ex

exec cargo watch -w crates -x "run --bin indexer"

