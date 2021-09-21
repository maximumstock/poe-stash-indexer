_init:
	docker-compose up -d

indexer-migrate: _init
	docker-compose exec indexer bash -c "diesel migration run"

indexer-start : _init
	docker-compose exec indexer indexer

logs:
	docker-compose logs -f --tail=20

up: _init

down:
	docker-compose down

restart:
	docker-compose restart $(CONTAINERS)

stop:
	docker-compose stop $(CONTAINERS)

build:
	docker build -t indexer:latest -f indexer/Dockerfile .

shell-indexer:
	docker-compose exec indexer bash

shell-db:
	docker-compose exec db bash

