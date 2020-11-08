![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

_Note: Very beta, breaking changes might occur_

This project focuses on building tooling around Path of Exile's Public Stash Tab API ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)).

Everything is written in [Rust](https://www.rust-lang.org/) (for fun) and is split into several subprojects.
Each subproject is its own crate in this workspace project:

- `indexer` - a bare-bones indexer that saves API river updates in a Postgres database
- `river-subscription` - a library for listening to the Stash Tab API river

## Indexer Features

- [x] Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- [x] Minimum indexing delay due to look-ahead for next `change_id` on partial HTTP response
- [x] Respect Stash Tab API rate limit
- [x] Persist stash updates in a PostgreSQL database in the forms of [Stash Records](indexer/migrations/2020-11-07-214742_stash_records/up.sql)
- [ ] Proper error handling in `river-subscription` with sensitive retry policy
- [ ] Configurable startup mode: resuming on last change_id vs latest updates
