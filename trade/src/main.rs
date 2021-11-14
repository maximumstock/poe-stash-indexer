mod api;
mod assets;
mod note_parser;
mod source;
mod store;

use std::sync::Arc;
use tokio::sync::Mutex;

use assets::AssetIndex;
use source::ExampleStream;
use store::Store;

/// TODO
/// [x] - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// [x] - filter currency items from `StashRecord`
///   - need asset mapping from pathofexile.com/trade
/// [ ] - will need state snapshots + restoration down the road
/// [ ] - RabbitMQ client that produces a stream of `StashRecord`s
/// [ ] - a web API that mimics pathofexile.com/trade API
/// [x] - note parsing to extract price
///       - look at https://github.com/maximumstock/poe-stash-indexer/blob/f7424546ffd40e1a74ecf6ca44584a74c2028957/src/parser.rs
///       - look at example stream to build note corpus -> sort -> unit test cases

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(Store::new("Scourge")));

    let stream = setup_inbounds(store.clone());
    let api = api::init(([127, 0, 0, 1], 3999), store);
    tokio::join!(stream, api);

    Ok(())
}

async fn setup_inbounds(store: Arc<Mutex<Store>>) {
    let mut asset_index = AssetIndex::new();
    asset_index.init().await.unwrap();

    let example_stream = ExampleStream::new("./data.json");

    for stash_record in example_stream {
        let mut store = store.lock().await;
        store.ingest_stash(stash_record, &asset_index);
        println!("Store has {:#?} offers", store.size());
        drop(store);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
