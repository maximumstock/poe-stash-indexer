# indexer

This crate defines the `indexer` part of this project, which listens to the
[Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes)
and lets you save its data to different [sinks](#sinks).

## Features

- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Emits stash updates as a stream of [Stash Records](indexer/src/stash_record.rs)

This generated around 670 GB of data for the first six weeks after Ancestor league start (2023-08-18 - 2023-09-30)
across all leagues, ie. all SC, all HC, and private leagues.

## Sinks

You can configure different sinks to pipe the indexed data to.
You can run zero or more sinks at any given time by configuring their respective environment variables.

- RabbitMQ - for further processing pipelines
- S3 - a bunch of timestamp partitioned `.jsonl` files

### RabbitMQ

The idea here is that `indexer` publishes whatever it finds under a pre-defined routing key,
which other services (eg. `trade-ingest` or something completely different) can consume to
build data pipelines.

#### Environemnt Variables

- `RABBITMQ_SINK_ENABLED=true|false|1|0` - to toggle the sink
- `RABBITMQ_URL` - a connection string to your RabbitMQ instance
- `RABBITMQ_PRODUCER_ROUTING_KEY` - the routing key to publish messages under

### S3

The idea here is to flush one minute-wide buffers of `StashRecord[]` as gzipped JSONL files
into a specified S3 bucket. Every minute, a new file in `{bucket-name}/{league}/{YYYY/mm/dd/HH/MM}.json.gz`
will be created, eg. `poe-stash-indexer/Ancestor/2023/08/23/12/34.json.gz`.

You are free to further process the data in whatever way you see fit.
AWS EMR/Glue and Athena could be used to compact the minute-wide chunks or run analytics on them.

#### Environment Variables

- `S3_SINK_ENABLED=true|false|1|0` - to toggle the sink
- `S3_BUCKET_NAME` - the name of the S3 bucket where the JSONL files will be stored
- `S3_REGION` - the AWS region where the S3 bucket is located
- `S3_SINK_ACCESS_KEY` & `S3_SINK_SECRET_KEY` - the AWS credentials to access the specified S3 bucket

## Error Handling

There a two types of errors to handle when running the indexer:

1. General network errors or unexpected API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is handled by waiting for the respective rate limit timeout internally and resuming once it is over.
