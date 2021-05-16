# stash-differ

A feature-extraction tool to generate features based on [`StashRecord`](../indexer/src/main.rs) diffs.

For running this tool, make sure the following index exists on `stash_records`.
This will take a while as our databases tend to be relatively large.

`create index chunk on stash_records using gist (league, int8range(id, id, '[]'))`
