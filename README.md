![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling around Path of Exile's Public Stash Tab API ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)).

Everything is written in [Rust](https://www.rust-lang.org/) and is split into several subprojects.
Each subproject is its own crate in this workspace project:

- `river-subscription` - a library for listening to the Stash Tab API river
- `indexer` - `river-subscription`-client that saves API river snapshots to a Postgres database
- `stash-differ` - a work-in-progress CLI tool to generate diff events between stash snapshots

You can run the indexer via `cargo run [--release]` from within the folder `indexer`.

## Features

- [x] Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Persists stash updates in a PostgreSQL database in the form of [Stash Records](indexer/migrations/2020-11-07-214742_stash_records/up.sql)

## Error Handling

There a two major error groups to handle:

1. General network errors or API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is not being handled by the indexer itself.
I've decided to handle this via service restarts on the server the indexer runs on - at least for now.
The reason being that internally waiting for the rate-limit to expire and then resuming work would result in inaccurate
`created_at` timestamps.
As accurate timestamps are a personal requirement of mine, handling rate-limit timeouts has to be performed externally.

---

**Update**
I've tested this indexer on AWS `us-ohio` server and could not notice any latency improvements.
Therefore the requirement for accurate timestamps is not realistic anymore.

---

**Note: Around 800 MB - 1 GB is generated per hour of indexing**
