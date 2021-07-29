_init: 
	docker-compose up -d

indexer-migrate: _init
	docker-compose run indexer bash -c "sleep 3 && diesel setup && diesel migration run"

indexer-start : _init
	docker-compose exec -it indexer indexer

logs:
	docker-compose logs -f --tail=20

up: _init

down: 
	docker-compose down

restart:
	docker-compose restart $(CONTAINERS)

stop:
	docker-compose stop $(CONTAINERS)