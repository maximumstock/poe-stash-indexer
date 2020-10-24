![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

A bare-bones API indexer for Path of Exile's Public Stash Tab API

## Features

- Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- Filters out all currency related items, ie. Orbs, Maps, Oils, Essences, Fragments, Catalysts, etc. 
  
  **NOTE**: At this point in time unique and rare items are not supported.
- Persists them in PostgreSQL database, running in container `poe`
