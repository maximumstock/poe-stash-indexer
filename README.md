![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling to gather and analyse data from Path of
Exile's [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)):

- `stash-api` - a library for listening to the Stash Tab API river
- [indexer](indexer/README.md) - saves API river snapshots to different sinks
- `stash-differ` - a work-in-progress tool to generate diff events between stash snapshots to create a player trading behaviour dataset

# How To Run

Our [`docker-compose.yaml`](./docker-compose.yaml) describes an examplatory setup of the [indexer](./indexer/README.md) that:

- saves collected data to a dockerized PostgreSQL database
- pushes collected data to a dockerized RabbitMQ exchange

This is meant to be run via your terminal or an equivalent [Docker](https://www.docker.com/) setup with `make` installed.
You can use the command `make up` to start up this configuration and
`make logs` to watch all logs.

Check out `Makefile` for more command aliases.

This pre-defined setup is only used for spontaneous testing and not meant to be copied and
ran in production.

Here is a list of services in this local development setup and and their credentials:

- [Prometheus](http://localhost:9090)
- [RabbitMQ Control Panel](http://localhost:15672) (`poe:poe`)
- [Grafana](http://localhost:3000) (`poe:poe`)
