UID := $(shell id -u)
GID := $(shell id -g)
docker-compose := env UID=${UID} GID=${GID} docker-compose

down:
	${docker-compose} down

up:
	${docker-compose} up -d

logs:
	${docker-compose} logs -f

migrate:
	${docker-compose} exec indexer diesel migration run

develop:
	up migrate

shell:
	${docker-compose} exec -it indexer /bin/bash

.PHONY: up down logs api-migrate develop shell
