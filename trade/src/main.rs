mod api;
mod assets;
mod note_parser;
mod source;
mod store;

use futures::StreamExt;
use lapin::options::BasicAckOptions;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{
    oneshot::{Receiver, Sender},
    Mutex,
};

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
/// [x] - validate offer results
/// [x] - RabbitMQ client that produces a stream of `StashRecord`s
/// [ ] - will need state snapshots + restoration down the road
/// [ ] - extend for multiple leagues
/// [ ] - a web API that mimics pathofexile.com/trade API
/// [x] - extend API response to contain number of offers as metadata
/// [ ] - pagination
/// [ ] - compression (its fine to do this server-side in this case)
/// [ ] - move from logs to metrics
///       - only log errors and debug info
///       - log and count unmappable item names
///       - metrics for all sorts of index sizes, number of offers, processed offers/service activity
/// [ ] - fix file paths

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    let store = runtime.block_on(setup_work(shutdown_rx, "Scourge".into()));

    println!("Saving store...");
    runtime.block_on(store.lock()).persist()?;

    println!("Shutting down");

    Ok(())
}

fn setup_shutdown_handler(signal_flag: Arc<AtomicBool>, shutdown_tx: Sender<()>) {
    std::thread::spawn(move || loop {
        if !signal_flag.load(Ordering::Relaxed) {
            continue;
        }

        shutdown_tx
            .send(())
            .expect("Signaling graceful shutdown failed");

        println!("Shutting down gracefully");
        return;
    });
}

async fn setup_work(shutdown_rx: Receiver<()>, league: String) -> Arc<Mutex<Store>> {
    let store = match Store::restore() {
        Ok(store) => {
            println!("Successfully restored store from file");
            store
        }
        Err(e) => {
            println!("Error restoring store: {:?}", e);
            let mut asset_index = AssetIndex::new();
            asset_index.init().await.unwrap();
            Store::new(league, asset_index)
        }
    };
    let store = Arc::new(Mutex::new(store));

    tokio::select! {
        _ = async {
            match setup_rabbitmq_consumer(shutdown_rx, store.clone()).await {
                Err(e) => eprintln!("Error setting up RabbitMQ consumer: {:?}", e),
                Ok(_) => println!("Initialized RabbitMQ consumer")
            }
        } => {},
        _ = api::init(([0, 0, 0, 0], 4001), store.clone()) => {},
    };

    store
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
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
    mut shutdown_rx: Receiver<()>,
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

        if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) =
            shutdown_rx.try_recv()
        {
            break;
        }
    }

    println!("Stopping RabbitMQ consumer");

    Ok(())
}
