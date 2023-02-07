# indexer

This crate defines the `indexer` part of this project, which listens to the
[Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes)
and lets you save its data to different [sinks](#sinks).

## Features

- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Emits stash updates as a stream of [Stash Records](indexer/src/stash_record.rs)
- [x] Lets you export said stream into [TimescaleDB](https://www.timescale.com/) and/or [RabbitMQ](https://www.rabbitmq.com/) for further processing

**Note: Around 800 MB - 1 GB is generated per hour of indexing during active play-time**

## Sinks

You can configure different sinks to pipe the indexed data to.
There are currently two types of sinks supported:

- TimescaleDB (a thin layer on top of PostgreSQL) - for persistent storage of raw API data
- RabbitMQ - for further processing pipelines

For using TimescaleDB set the `DATABASE_URL` environment variable to a valid PostgreSQL connection string.

For using RabbitMQ set the following environment variables:

- `RABBITMQ_SINK_ENABLED=true|false|1|0` - to toggle the sink
- `RABBITMQ_URL` - a connection string to your RabbitMQ instance
- `RABBITMQ_PRODUCER_ROUTING_KEY` - the routing key to publish messages under

## Error Handling

There a two types of errors to handle when running the indexer:

1. General network errors or unexpected API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is handled by waiting for the respective rate limit timeout internally and resuming once it is over.
