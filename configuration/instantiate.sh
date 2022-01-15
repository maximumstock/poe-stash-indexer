#!/usr/bin/env bash

set -e
cp -r templates/* instantiated
set -o allexport;

case $ENV in
  "production")
    echo "$AGE_KEY" > key.txt
    age --decrypt -i key.txt -o environments/.env.production environments/.env.production.enc
    rm key.txt
    source environments/.env.production;
    ;;
  *)
    source environments/.env.development;
    ;;
esac

set +o allexport

for f in $(find instantiated -type f); do
  SUBS=$(cat $f | envsubst "$(env | cut -d= -f1 | sed -e 's/^/$/')")
  echo "$SUBS" > $f
  echo "Substituted $f"
done
