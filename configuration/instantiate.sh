#!/usr/bin/env bash

set -e

cp -r templates instantiated

# todo generalize
set -o allexport; source environments/.env.development; set +o allexport

for f in $(find instantiated -type f); do
  SUBS=$(cat $f | envsubst "$(env | cut -d= -f1 | sed -e 's/^/$/')")
  echo "$SUBS" > $f
  echo "Substituted $f"
done
