# trade-ingest

Separate service that consumes a stream of [Stash Records](../indexer/src/stash_record.rs) from `indexer` via RabbitMQ
and ingests all currency item trading offers into a PostgreSQL database for [trade-api](../trade-api) to serve.
