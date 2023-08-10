![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling to gather and analyse data from Path of
Exile's [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)) and consists of the following crates:

- [stash-api](crates/stash-api/README.md) - a library for consuming the [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) river
- [indexer](crates/indexer/README.md) - a service that saves API river snapshots to different sinks like PostgreSQL & RabbitMQ
- [trade-ingest](crates/trade-ingest/README.md) - a service that feeds data from `indexer` & RabbitMQ into a PostgreSQL (TimescaleDB) instance
- [trade-api](crates/trade-api/README.md) - a service exposes a REST-like API to query player's trading offers from said PostgreSQL instance
- [stash-differ](crates/stash-differ/README.md) - a work-in-progress cli tool to generate diff events between stash snapshots to create a player trading behaviour dataset

## Service Architecture

A brief overview over what is going on:

![img](docs/architecture.svg)

## Public Services

`trade-api` exposes a public REST-like API whose documentation you can find [here](crates/trade-api/README.md).

## Quickstart

Generally the workflow for running this project is:

1. Create an `.env.development` from `.env.template`
2. `make config` to instantiate all service configurations based on `.env.development`
3. `make build` to build Docker images, as currently still necessary for some parts
4. `make up`, starting everything as declared in `docker-compose.yml`

For a separate production setup, change the first steps:

1. Create `.env.production` as you want
2. Run `ENV=production make config`
3. Run `make build-prod` to build production Docker images
4. Run `make up-prod` to start production Docker containers

## Local Dev Environment

Our [`docker-compose.yaml`](./docker-compose.yaml) describes an examplatory (non-production) setup of the above tools for experimentation, which includes:

- setup of the `indexer` service to start fetching and feed the data stream into a RabbitMQ instance
- setup of the `trade-ingest` & `trade-api` services to consume above stream and expose it via its REST-like API, respectively.
- a PostgreSQL instance
- a [OTLP setup](https://github.com/open-telemetry/opentelemetry-rust) (integrating with New Relic) to investigate metrics of the `indexer`, `trade-ingest`, `trade-api` and RabbitMQ services
- exposes a reverse proxy setup via nginx to easily access all services

You may execute `make up` to start everything up and `make logs` to watch all logs.
Check out `Makefile` for more command aliases.

Here is a list of services in this local development setup and and their credentials (`username`:`password`):

- [Trade API](http://trade.localhost:8888) (public)
- [RabbitMQ Control Panel](http://rabbitmq.localhost:8888) (Basic Auth: `poe:poe`)
