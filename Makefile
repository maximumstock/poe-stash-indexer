UID := $(shell id -u)
GID := $(shell id -g)
docker-compose := env UID=${UID} GID=${GID} docker-compose

down:
	${docker-compose} down

up:
	${docker-compose} up -d

logs:
	${docker-compose} logs -t -f --tail 50

migrate:
	${docker-compose} exec indexer diesel migration run

develop:
	up migrate

shell:
	${docker-compose} exec indexer /bin/bash

db-shell:
	${docker-compose} exec db /bin/bash

.PHONY: up down logs api-migrate develop shell db-shell
