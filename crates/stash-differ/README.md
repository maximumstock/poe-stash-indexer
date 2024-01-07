# stash-differ

A feature-extraction tool to generate features based on [`StashRecord`](../indexer/src/stash_record.rs) diffs.

It scans through all chunks that the Stash Tab API emits and keeps track of how
each stash of each player changes over time.

From there, we generate events like:

- ItemAdded
- ItemRemoved
- ItemNoteChanged
- ItemStackSizeChanged

to track player activity on an abstract level and persist them as CSV.
