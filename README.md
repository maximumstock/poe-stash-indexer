[![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/poe-stash-indexer/actions/workflows/rust.yml)

# poe-stash-indexer

_Keep in mind: As there is no public stash tab API for PoE 2 yet, this project only works for PoE 1._
_Once support for a PoE 2 stash tab API is available, I will work on it._

This project aims to build tooling to gather data from Path of Exile's
[Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes)
(see also the older [community wiki documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)).

The main component in this project is the [indexer](crates/indexer/README.md) application.
It offers an easy way to consume the [Public Stash Tab API river](https://www.pathofexile.com/developer/docs/reference#publicstashes) and flush it data sinks like RabbitMQ or S3 for further processing.

More on the installation & usage in the dedicated [documentation](./crates/indexer/README.md).

## Project Structure

```bash
├── Makefile
├── README.md       # you are here
├── crates
│   ├── indexer     # the main application
│   ├── stash-api   # an internal library that `indexer` uses below are some internal prototypes, you can ignore for now
│   ├── stash-differ
│   ├── trade-api
│   ├── trade-common
├── notes           # some notes for myself
├── infra           # internal scripts, CI parts and documentation for my own `indexer` deployment
└── shell.nix
```

## Local Dev Environment

If you are using `nix`, just run `nix-shell` and enjoy all dependencies being installed.

From there you can setup the respective crate application you want to run via environment variables however you see fit.
See the [indexer README.md](./crates/indexer/README.md) for more information on what environment variables to set.

### Example Docker Setup

There are a couple different applications in this umbrella project of which only `indexer` is as of now relevant for users.
As an example, there is [`infra/docker-compose.yaml`](./docker-compose.yaml) defines a setup of `indexer` and some of the other prototypes.
Feel free to use or copy any of it.

- `indexer` - setup of the `indexer` service to start fetching and feed the data stream into a RabbitMQ instance
- `rabbitmq` - a RabbitMQ instance for `indexer` to ingest new `Stash` batches into to test the RabbmitMQ sink
- setup of the `trade-ingest` & `trade-api` services to consume above stream and expose it via its REST-like API, respectively
- `trade-store` - a PostgreSQL instance for `trade-ingest` to ingest into and `trade-api` to read data from
- `otel-collector` - an [OTLP setup](https://github.com/open-telemetry/opentelemetry-rust) (integrating with New Relic) to investigate metrics of the `indexer`, `trade-ingest`, `trade-api`
- `reverse-proxy` - exposes a reverse proxy setup via nginx to easily access all services (see below)

Here is a list of services in this local development setup and and their credentials (`username`:`password`):

- [Trade API](http://trade.localhost:8888) (public)
- [RabbitMQ Control Panel](http://rabbitmq.localhost:8888) (Basic Auth: `poe:poe`)
- [Jaeger Dashboard](http://jaeger.localhost:8888) (public)

For testing the PostgreSQL `indexer` sink, you can run something like:

`docker run --name pg-poe -e POSTGRES_PASSWORD=poe -e POSTGRES_USER=poe -e POSTGRES_DB=poe -p 5432:5432 postgres:latest`

and connect to `pg://poe:poe@127.0.0.1:5432/poe`.
