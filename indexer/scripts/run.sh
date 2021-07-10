#!/usr/bin/env bash

while !</dev/tcp/db/5432; do sleep 1; done;

diesel setup
diesel migration run

cargo watch -x run