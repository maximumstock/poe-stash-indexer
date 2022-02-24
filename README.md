![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling to gather and analyse data from Path of
Exile's [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)) and consists of the following crates:

- [stash-api](stash-api/README.md) - a library for consuming the [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) river
- [indexer](indexer/README.md) - a service that saves API river snapshots to different sinks like PostgreSQL & RabbitMQ
- [trade](trade/README.md) - a service that feeds in data from `indexer` and exposes a REST-like API to query player's trading offers
- [stash-differ](stash-differ/README.md) - a work-in-progress cli tool to generate diff events between stash snapshots to create a player trading behaviour dataset

# How To Run

Our [`docker-compose.yaml`](./docker-compose.yaml) describes an examplatory (non-production) setup of the above tools for experimentation, which includes:

- setup of the `indexer` service to start fetching and feed the data stream into a RabbitMQ instance
- setup of the `trade` service to consume above stream and expose it via its REST-like API
- a currently unused PostgreSQL instance that can be used as an data sink alternative to RabbitMQ
- a Grafana & Prometheus setup to investigate metrics of the `indexer`, `trade` and RabbitMQ services
- exposes a reverse proxy setup via nginx to easily access all services

You may execute `make up` to start everything up and `make logs` to watch all logs. Check out `Makefile` for more command aliases.

Here is a list of services in this local development setup and and their credentials (`username`:`password`):

- [Indexer Metrics Endpoint/Healthcheck](http://indexer.localhost:8888)
- [Trade API](http://trade.localhost:8888)
- [Prometheus](http://prometheus.localhost:8888)
- [RabbitMQ Control Panel](http://rabbitmq.localhost:8888) (`poe:poe`)
- [Grafana](http://grafana.localhost:8888) (`poe:poe`)
