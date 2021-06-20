![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling around Path of Exile's Public Stash Tab API
([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)).

- `river-subscription` - a library for listening to the Stash Tab API river
- `indexer` - `river-subscription`-client that saves API river snapshots to a Postgres database
- `stash-differ` - a work-in-progress CLI tool to generate diff events between stash snapshots

## Indexer Features

- [x] Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respects Stash Tab API [rate limit](https://pathofexile.gamepedia.com/Public_stash_tab_API#Rate_Limit)
- [x] Persists stash updates in a PostgreSQL database in the form of [Stash Records](indexer/src/stash_record.rs)

**Note: Around 800 MB - 1 GB is generated per hour of indexing**

## Error Handling

There a two types of errors handle:

1. General network errors or API server errors
2. Running into rate-limit timeouts

The former is being handled by naively rescheduling requests in hope the error resolves itself.
The indexer exits after three unsuccessful tries for the same change id.

The latter is not being handled by the indexer itself.
I've decided to handle this via service restarts on the server the indexer runs on - at least for now.
