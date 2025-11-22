# indexer

This crate defines the `indexer` part of this project, which listens to the
[Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes)
and lets you save its data to different [sinks](#sinks).

## Features

- [x] Collects stash updates as a stream of [`Stash`](../stash-api/src/common/stash.rs) via the [`stash-api`](../stash-api/README.md) crate
- [x] Export data to different [sinks](#sinks)
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Graceful handling of shutdown signals by flushing all sinks

## Prerequisites

You will need valid developer credentials to authenticate `indexer` with GGG.
The [official API documentation](https://www.pathofexile.com/developer/docs/index#gettingstarted) describes
what you need to do in order to receive such credentials.

Make sure your credentials fulfill the following requirements:

- allow the [`Confidential Clients` client type](https://www.pathofexile.com/developer/docs/authorization#clients-confidential)
- allow the [`Client Credentials Grant` grant type](https://www.pathofexile.com/developer/docs/authorization#grants-client-credentials)
- permit the [`service:psapi` API scope](https://www.pathofexile.com/developer/docs/authorization#scopes)

## Installation & Quickstart

You can either build and install the application yourself by:

```bash
git clone https://github.com/maximumstock/poe-stash-indexer
cargo install --path crates/indexer
```

or use the latest Docker image:

```bash
# Only published platform is linux/amd64 as of now, but you can always build the Dockerfile yourself for your platform of choice
docker run --platform linux/amd64 \
    # these configuration options are required to talk to the API
    # see the configuration options below
    -e POE_CLIENT_ID="" \
    -e POE_CLIENT_SECRET="" \
    -e POE_DEVELOPER_MAIL="" \
    maximumstock2/indexer:latest
```

## Configuration

Here is a list of all available environment variable configuration options.

The required Path of Exile API credentials can be obtained by requesting an account through GGG, as
described in [their API documentation](https://www.pathofexile.com/developer/docs/index#gettingstarted).

| Environment Variable            | Required                             | Default             | Description                                                                   |
| ------------------------------- | ------------------------------------ | ------------------- | ----------------------------------------------------------------------------- |
| `POE_CLIENT_ID`                 | yes                                  |                     | Your personal Path of Exile API client id                                     |
| `POE_CLIENT_CLIENT_SECRET`      | yes                                  |                     | Your personal Path of Exile API client secret key                             |
| `POE_DEVELOPER_EMAIL`           | yes                                  |                     | A contact email for GGG to contact if the linked API account misbehaves       |
| `RESTART_MODE`                  | no                                   | "fresh"             | See [Stopping & Resuming](#stopping--resuming) for more information           |
| `RABBITMQ_SINK_ENABLED`         | no                                   | false               | To toggle the sink                                                            |
| `RABBITMQ_URL`                  | if `RABBITMQ_SINK_ENABLED` is `true` |                     | The connection string to your RabbitMQ instance                               |
| `RABBITMQ_PRODUCER_ROUTING_KEY` | no                                   | "poe-stash-indexer" | The routing key to publish messages under                                     |
| `POSTGRES_SINK_ENABLED`         | no                                   | false               | To toggle the sink                                                            |
| `POSTGRES_URL`                  | if `POSTGRES_SINK_ENABLED` is `true` |                     | The connection string to your PostgreSQL instance                             |
| `S3_SINK_ENABLED`               | no                                   | false               | To toggle the sink                                                            |
| `S3_SINK_BUCKET_NAME`           | if `S3_SINK_ENABLED" is `true`       |                     | The name of the S3 bucket where the JSONL files will be stored                |
| `S3_SINK_REGION`                | no                                   |                     | The AWS region where the S3 bucket is located                                 |
| `OTEL_COLLECTOR`                | no                                   |                     | The gRPC endpoint of an OTEL collector sidecar daemon, collecting OTLP traces |

## Sinks

The `indexer` crate uses the [`stash-api`](../stash-api/README.md) crate to collect stash updates from the official
Path of Exile API and transforms it into a stream of [`Stash`](../stash-api/src/common/stash.rs) records.

You can run zero or more sinks at any given time by configuring their respective environment variables.

Implemented:

- [x] [RabbitMQ](#rabbitmq) - for further processing pipelines
- [x] [S3](#s3) - a bunch of timestamp partitioned `.jsonl` files in JSONL format

In Progress:

- [PostgreSQL](#postgresql)
- [JSON file](#local-file) - exporting the stream directly to a local file in JSON format for quicker prototyping
- [Kafka](#kafka)

Each sink was created with a certain idea and use-case in mind.
See below to find out more on each sink design and what data format to expect.

### RabbitMQ

The idea here is that `indexer` publishes whatever it finds under a (customisable) routing key, which other services
(eg. `trade-ingest` or something completely different) can consume to build data pipelines.

In terms of data format, this sink sends messages with JSON array of the raw [`Stash`](../stash-api/src/common/stash.rs) update.

### S3

The idea here is to flush one minute-wide arrays of [`Stash`](../stash-api/src/common/stash.rs) as gzipped JSONL files
into a specified S3 bucket. Every minute, a new file in `{bucket-name}/{league}/{YYYY/mm/dd/HH/MM}.json.gz`
will be created, eg. `poe-stash-indexer/Ancestor/2023/08/23/12/34.json.gz`.

By default, the AWS Rust SDK reads your environment variables to find AWS credentials and picks up your credentials & region, but you can always override the latter via `S3_SINK_REGION`.
So if you use your AWS CLI locally to create AWS credentials for your shell session and export these environment variables, the AWS SDK and `indexer` will automatically pick up your credentials.
If you use SSO via your AWS CLI then you might have to set the environment variable `AWS_PROFILE` to specify the correct credential SSO profile, ie. `AWS_PROFILE="my-profile" cargo run --bin indexer`.

You are free to further process the data in whatever way you see fit.
AWS EMR/Glue and Athena could be used to compact the minute-wide chunks or run analytics on them.

### PostgreSQL

tbd

### Local File

tbd

### Kafka

tbd

## Stopping & Resuming

When stopping `indexer` (sending `SIGINT` or `SIGTERM` e.g. via your CLI, `top` or `systemd`), it flushes some state to
`./indexer_state.json` in its local directory on disk.
This file contains metadata so `indexer` knows where it left off when it was stopped the last time.

By default, when you start `indexer` again, it uses `RestartMode::Fresh` and fetches the latest change id [poe.ninja](https://poe.ninja/)
and therefore might skip the change id between when you left off and when you restart `indexer`.

If you want to force `indexer` to pick up where it left off you can enable `RestartMode::Resume` by setting the environment variable `RESTART_MODE=resume`.
With this you will make sure to traverse all change ids in order, but you might not catch up to the latest data on the stream and be continuously behind.

I recommend just using the defaults unless you specifically are fine with scraping out-of-date data.

## Error Handling

There a two types of errors to handle when running the indexer:

1. General network errors or unexpected API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is handled by waiting for the respective rate limit timeout internally and resuming once it is over.
