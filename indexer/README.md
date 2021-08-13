# indexer

This crate defines the `indexer` part of this project, which listens to the
[Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes)
and lets you save its data to different [sinks](#sinks).

## Features

- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Persists stash updates in a PostgreSQL database in the form of [Stash Records](indexer/src/stash_record.rs)

**Note: Around 800 MB - 1 GB is generated per hour of indexing during active play-time**

## Sinks

You can configure different sinks to pipe the indexed data to.
There are currently two types of sinks supported:

- PostgreSQL - for persistent storage of raw API data
- RabbitMQ - for further processing pipelines

By default, the indexer pipes the collected data into a configured PostgreSQL instance, which
you can specify via setting the `DATABASE_URL` environment variable to a valid PostgreSQL
connection string.
This behaviour is not configurable right now and you are expected to provide this configuration.

Optionally, you can configure RabbitMQ by setting the following environment variables:

- `RABBITMQ_SINK_ENABLED=true|1` - to toggle the sink
- `RABBITMQ_URL` - a connection string to your RabbitMQ instance
- `RABBITMQ_PRODUCER_ROUTING_KEY` - the routing key to publish messages under

## Error Handling

There a two types of errors to handle when running the indexer:

1. General network errors or unexpected API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is not being handled by the indexer itself.
I've decided to handle this via service restarts of the indexer - at least for now.
