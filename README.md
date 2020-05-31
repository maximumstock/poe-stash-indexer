# poe-stash-indexer
A bare-bones API indexer for Path of Exile's Public Stash Tab API

## Features
- Listens on the [Public Stash Tab API](https://www.pathofexile.com/api/public-stash-tabs) river stream
- Filters out all currency related items, ie. Orbs, Maps, Oils, Essences, Fragments, Catalysts, etc.
- Persists them in a simple JSON-formatted `db.json`
