#!/usr/bin/env bash

set -e
mkdir -p instantiated
cp -r templates/* instantiated
set -o allexport;

echo "$AGE_KEY" > key.txt
case $ENV in
  "production")
    age --decrypt -i key.txt -o environments/.env.production environments/.env.production.enc
    source environments/.env.production;
    ;;
  *)
    age --decrypt -i key.txt -o environments/.env.development environments/.env.development.enc
    source environments/.env.development;
    ;;
esac
rm key.txt

set +o allexport

for f in $(find instantiated -type f); do
  SUBS=$(cat $f | envsubst "$(env | cut -d= -f1 | sed -e 's/^/$/')")
  echo "$SUBS" > $f
  echo "Substituted $f"
done
