![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling to gather and analyse data from Path of Exile's [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)).

## Use-Cases

1. Do you need a service to scrape the [Public Stash Tab API river](https://www.pathofexile.com/developer/docs/reference#publicstashes)?

Use [indexer](crates/indexer/README.md) either self-compiled or as a pre-packaged Docker image.

2. Do you want to programmatically scrape the [Public Stash Tab API river](https://www.pathofexile.com/developer/docs/reference#publicstashes)?

Use [stash-api](crates/stash-api/README.md) to programmatically consume

- [indexer](crates/indexer/README.md) - a service that saves API river snapshots to different sinks like RabbitMQ or S3
- [stash-api](crates/stash-api/README.md) - a library for consuming the [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) river

## Local Dev Environment

Our [`docker-compose.yaml`](./docker-compose.yaml) defines a setup of the above tools for experimentation, which includes:

- `indexer` - setup of the `indexer` service to start fetching and feed the data stream into a RabbitMQ instance
- `rabbitmq` - a RabbitMQ instance for `indexer` to ingest new `StashRecord` batches into and `trade-ingest` to read data from
- setup of the `trade-ingest` & `trade-api` services to consume above stream and expose it via its REST-like API, respectively
- `trade-store` - a PostgreSQL instance for `trade-ingest` to ingest into and `trade-api` to read data from
- `otel-collector` - an [OTLP setup](https://github.com/open-telemetry/opentelemetry-rust) (integrating with New Relic) to investigate metrics of the `indexer`, `trade-ingest`, `trade-api`
- `reverse-proxy` - exposes a reverse proxy setup via nginx to easily access all services (see below)

Here is how you can run it:

1. Create an `configuration/environments/.env.development` from `.env.template` with your credentials
2. `make config` to instantiate all service configurations based on `.env.development`
3. `make build` to build Docker images, as currently still necessary for some parts (note: this might take some time initially)
4. `make up`, starting everything as declared in `docker-compose.yml`
5. `make down`, stopping everything as declared in `docker-compose.yml`

When you change the environment variables, go back to step 2.

`make logs` lets you inspect all Docker container logs. Check out `Makefile` for more command aliases.

Here is a list of services in this local development setup and and their credentials (`username`:`password`):

- [Trade API](http://trade.localhost:8888) (public)
- [RabbitMQ Control Panel](http://rabbitmq.localhost:8888) (Basic Auth: `poe:poe`)
- [Jaeger Dashboard](http://jaeger.localhost:8888) (public)
