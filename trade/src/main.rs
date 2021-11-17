mod api;
mod assets;
mod note_parser;
mod source;
mod store;

use futures::StreamExt;
use lapin::options::BasicAckOptions;
use std::sync::Arc;
use tokio::sync::Mutex;

use assets::AssetIndex;
use source::{ExampleStream, StashRecord};
use store::Store;

use crate::source::setup_consumer;

/// TODO
/// [x] - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// [x] - filter currency items from `StashRecord`
///   - need asset mapping from pathofexile.com/trade
/// [x] - note parsing to extract price
///       - look at https://github.com/maximumstock/poe-stash-indexer/blob/f7424546ffd40e1a74ecf6ca44584a74c2028957/src/parser.rs
///       - look at example stream to build note corpus -> sort -> unit test cases
/// [x] - created_at timestamp on offers
/// [ ] - validate offer results
/// [ ] - RabbitMQ client that produces a stream of `StashRecord`s
/// [ ] - will need state snapshots + restoration down the road
/// [ ] - extend for multiple leagues
/// [ ] - a web API that mimics pathofexile.com/trade API
/// [ ] - extend API response to contain number of offers as metadata
/// [ ] - pagination
/// [ ] - compression (its fine to do this server-side in this case)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut asset_index = AssetIndex::new();
    asset_index.init().await.unwrap();

    let store = Arc::new(Mutex::new(Store::new("Scourge", asset_index)));

    // let stream = setup_local_consumer(store.clone());
    let stream = setup_rabbitmq_consumer(store.clone());
    let api = api::init(([0, 0, 0, 0], 4001), store);
    let x = tokio::join!(stream, api);
    x.0.unwrap();

    Ok(())
}

#[allow(dead_code)]
async fn setup_local_consumer(store: Arc<Mutex<Store>>) {
    let example_stream = ExampleStream::new("./data.json");

    for stash_record in example_stream {
        let mut store = store.lock().await;
        store.ingest_stash(stash_record);
        println!("Store has {:#?} offers", store.size());
        drop(store);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn setup_rabbitmq_consumer(
    store: Arc<Mutex<Store>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut consumer = setup_consumer().await?;

    while let Some(incoming) = consumer.next().await {
        let (_, delivery) = incoming.unwrap();
        delivery.ack(BasicAckOptions::default()).await?;

        let stash_records = serde_json::from_slice::<Vec<StashRecord>>(&delivery.data)?;

        let mut store = store.lock().await;
        for stash_record in stash_records {
            store.ingest_stash(stash_record);
        }
        println!("Store has {:#?} offers", store.size());
    }

    Ok(())
}
