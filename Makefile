dc := docker-compose -f docker-compose.yaml
dc-prod := docker-compose -f docker-compose.yaml -f docker-compose.production.yaml

init:
	$(dc) up -d
init-prod:
	$(dc-prod) up -d

up: build init
up-prod: build-prod init-prod

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
	$(dc) logs -f --tail=20


# Indexer targets
indexer-migrate: _init
	$(dc) exec indexer bash -c "diesel setup"

indexer-start : _init
	$(dc) exec indexer indexer

shell-indexer:
	$(dc) exec indexer bash

shell-db:
	$(dc) exec db bash
