#!/bin/env sh
docker-compose exec trade-store pg_dump -Upoe > pg_dump.sql
