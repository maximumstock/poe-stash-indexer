# stash-api

A Rust library crate for efficient consumption of [Path of Exile's](https://pathofexile.com) [Public Stash Tab API](https://www.pathofexile.com/developer/docs/reference#publicstashes).

## Features

- Efficient look-ahead parsing of partial response bodies so we can queue the next chunk as soon as possible
- Fetches latest change ids from [poe.ninja](https://poe.ninja)
- Threaded architecture & small dependency footprint by preferring a blocking API over async

## Usage

```rs
let mut indexer = Indexer::new();

// You can start consuming the stream starting at the latest publicly available chunk...
let rx = indexer.start_with_latest();
// ...or start with a pre-defined chunk.
// let rx = indexer.start_with_id(ChangeId::from_str(&str).unwrap())

// `Indexer` currently only offers a blocking API via a `std::mpsc::channel`.
// Matching on `IndexerMessage` let's you react accordingly.
while let Ok(msg) = rx.recv() {
    match msg {
        // The `Stop` variant is emitted if someone calls `indexer.stop()` and all meanwhile
        // fetched chunks are done processing.
        IndexerMessage::Stop => break,
        IndexerMessage::RateLimited(timer) => {
            log::info!("Rate limited for {} seconds...waiting", timer.as_secs());
        }
        IndexerMessage::Tick {
            change_id,
            payload,
            created_at,
        } => {
            log::info!(
                "Processing {} ({} stashes)",
                change_id,
                payload.stashes.len()
            );

            let next_change_id = payload.next_change_id.clone();
            log::info!("The next change id: {}", next_change_id);
        }
    }
}
```
