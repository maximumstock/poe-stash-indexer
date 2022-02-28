mod api;
mod assets;
mod consumer;
mod note_parser;
mod source;
mod store;

use metrics::Metrics;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{
    oneshot::{Receiver, Sender},
    Mutex,
};

use assets::AssetIndex;
use store::Store;

use crate::metrics::setup_metrics;

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
/// [x] - will need state snapshots + restoration down the road
/// [x] - fix file paths
/// [ ] - extend for multiple leagues
/// [ ] - a web API that mimics pathofexile.com/trade API
/// [x] - extend API response to contain number of offers as metadata
/// [ ] - add proper logging
/// [ ] - pagination
///       - [x] limit query parameter
/// [x] - compression (its fine to do this server-side in this case)
/// [ ] - move from logs to metrics
///       - only log errors and debug info
///       - log and count unmappable item names
///       - metrics for all sorts of index sizes, number of offers, processed offers/service activity
mod metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    let store = Arc::new(Mutex::new(load_store("Scourge".into()).await.unwrap()));
    let metrics = setup_metrics(std::env::var("METRICS_PORT")?.parse()?)?;
    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    let store = setup_work(shutdown_rx, store, metrics).await;

    println!("Saving store...");
    store.lock().await.persist()?;
    println!("Shutting down");

    Ok(())
}

fn setup_shutdown_handler(signal_flag: Arc<AtomicBool>, shutdown_tx: Sender<()>) {
    std::thread::spawn(move || loop {
        if !signal_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        shutdown_tx
            .send(())
            .expect("Signaling graceful shutdown failed");

        println!("Shutting down gracefully");
        break;
    });
}

async fn setup_work(
    shutdown_rx: Receiver<()>,
    store: Arc<Mutex<Store>>,
    metrics: impl Metrics + Clone + Send + Sync + 'static,
) -> Arc<Mutex<Store>> {
    let metrics2 = metrics.clone();

    tokio::select! {
        _ = async {
            match consumer::setup_rabbitmq_consumer(shutdown_rx, store.clone(), metrics).await {
                Err(e) => eprintln!("Error setting up RabbitMQ consumer: {:?}", e),
                Ok(_) => println!("Consumer decomissioned")
            }
        } => {},
        _ = api::init(([0, 0, 0, 0], 4001), store.clone(), metrics2) => {},
    };

    store
}

async fn load_store(league: String) -> Result<Store, Box<dyn std::error::Error>> {
    let store = match Store::restore() {
        Ok(store) => {
            println!("Successfully restored store from file");
            store
        }
        Err(e) => {
            eprintln!("Error restoring store, creating new: {:?}", e);
            let mut asset_index = AssetIndex::new();
            asset_index.init().await?;
            let store = Store::new(league, asset_index);
            store.persist()?;
            store
        }
    };
    Ok(store)
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
