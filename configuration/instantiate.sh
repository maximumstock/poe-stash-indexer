#!/usr/bin/env bash

set -e
mkdir -p instantiated
cp -r templates/* instantiated

if [ ! -z ${AGE_KEY+x} ]; then
  echo "$AGE_KEY" > key.txt
  age --decrypt -i key.txt -o environments/.env.development environments/.env.development.enc
  age --decrypt -i key.txt -o environments/.env.production environments/.env.production.enc
  rm key.txt
fi

set -o allexport;
case $ENV in
  "production")
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
