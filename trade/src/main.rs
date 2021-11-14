use std::sync::Arc;

use source::ExampleStream;
use store::Store;
use tokio::sync::Mutex;

use crate::assets::AssetIndex;

mod api;
mod assets;
mod source;
mod store;
/// TODO
/// [ ] - RabbitMQ client that produces a stream of `StashRecord`s
/// [x] - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// [ ] - a web API that mimics pathofexile.com/trade API
/// [ ] - will need state snapshots + restoration down the road
/// [x] - filter currency items from `StashRecord`
///   - need asset mapping from pathofexile.com/trade
/// [ ] - note parsing to extract price

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
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
