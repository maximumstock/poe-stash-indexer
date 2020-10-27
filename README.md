![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

A bare-bones API indexer for Path of Exile's Public Stash Tab API [Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API).

## Features

- Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- Looks for all currency related items, ie. Orbs, Maps, Oils, Essences, Fragments, Catalysts, etc.

  **NOTE**: At this point in time unique and rare items are not supported.

- Persists them in a PostgreSQL database in the forms of [Offers](https://github.com/maximumstock/poe-stash-indexer/blob/master/migrations/2020-06-11-182409_offers/up.sql#L2)
