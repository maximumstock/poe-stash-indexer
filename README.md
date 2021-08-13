![Continuous Integration](https://github.com/maximumstock/poe-stash-indexer/workflows/Continuous%20Integration/badge.svg)

# poe-stash-indexer

This project focuses on building tooling to gather and analyse data from Path of
Exile's [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes) ([Wiki Documentation](https://pathofexile.gamepedia.com/Public_stash_tab_API)):

- `river-subscription` - a library for listening to the Stash Tab API river
- [indexer](indexer/README.md) - saves API river snapshots to different sinks
- `stash-differ` - a work-in-progress tool to generate diff events between stash snapshots to create a player trading behaviour dataset
