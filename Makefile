# UID := $(shell id -u)
# GID := $(shell id -g)

#docker-compose := env UID=${UID} GID=${GID} docker-compose
docker-compose := docker-compose

dc := ${docker-compose} -f docker-compose.yaml --env-file configuration/environments/.env.development
dc-prod := ${docker-compose} -f docker-compose.yaml -f docker-compose.production.yaml --env-file configuration/environments/.env.production

# Sets up all configuration file templates in `configuration/templates` by copying them to
# `configuration/instantiated` and populating all environment variable references with values
# from the respective `configuration/environments/.env.{development,production}` spec.
config:
	cd configuration && ./instantiate.sh
encrypt:
	age --encrypt -i secrets/age.key -o configuration/environments/.env.development.enc configuration/environments/.env.development
	age --encrypt -i secrets/age.key -o configuration/environments/.env.production.enc configuration/environments/.env.production
decrypt:
	age --decrypt -i secrets/age.key -o configuration/environments/.env.development configuration/environments/.env.development.enc
	age --decrypt -i secrets/age.key -o configuration/environments/.env.production configuration/environments/.env.production.enc

init:
	$(dc) up -d --remove-orphans
init-prod:
	$(dc-prod) up -d --remove-orphans

up: init
up-prod: init-prod

build:
	$(dc) build --force-rm $(CONTAINERS)
build-prod:
	$(dc-prod) build --force-rm $(CONTAINERS)

down:
	$(dc) down --remove-orphans

restart:
	$(dc) restart $(CONTAINERS)

stop:
	$(dc) stop $(CONTAINERS)

logs:
	${dc} logs -f -t --tail=20

tidy:
	cargo fmt --all -- --check && cargo clippy -- -D warnings

test:
	cargo test --all-features -- --nocapture

# Indexer targets
indexer-migrate: init
	$(dc) exec indexer bash -c "diesel setup"

indexer-start : init
	$(dc) exec indexer indexer

shell-indexer:
	$(dc) exec indexer bash

shell-db:
	$(dc) exec trade-store bash
