UID := $(shell id -u)
GID := $(shell id -g)

docker-compose := env UID=${UID} GID=${GID} docker-compose

dc := ${docker-compose} -f docker-compose.yaml
dc-prod := ${docker-compose} -f docker-compose.yaml -f docker-compose.production.yaml

init:
	$(dc) up --build -d
init-prod:
	$(dc-prod) up --build -d

up: init
up-prod: init-prod

build:
	$(dc) build --force-rm indexer
build-prod:
	$(dc-prod) build --force-rm indexer

down:
	$(dc) down --remove-orphans

restart:
	$(dc) restart $(CONTAINERS)

stop:
	$(dc) stop $(CONTAINERS)

logs:
	${docker-compose} logs -f --tail=20

tidy:
	cargo fmt --all -- --check && cargo clippy -- -D warnings

# Indexer targets
indexer-migrate: _init
	$(dc) exec indexer bash -c "diesel setup"

indexer-start : _init
	$(dc) exec indexer indexer

shell-indexer:
	$(dc) exec indexer bash

shell-db:
	$(dc) exec db bash
