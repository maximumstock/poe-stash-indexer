mod api;
mod assets;
mod config;
mod consumer;
mod league;
mod metrics;
mod note_parser;
mod source;
mod store;

use config::Config;
use league::League;
use metrics::store::StoreMetrics;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{
    oneshot::{Receiver, Sender},
    RwLock,
};

use tracing::{error, info};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

use store::StoreMap;

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
/// [x] - add proper logging
/// [x] - pagination
///       - [x] limit query parameter
/// [x] - compression (its fine to do this server-side in this case)
/// [-] - move from logs to metrics + traces
///       - only log errors and debug info
///       - log and count unmappable item names
///       - metrics for all sorts of index sizes, number of offers, processed offers/service activity

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;

    setup_tracing().expect("Tracing setup failed");
    let store_map = setup_store_map().await?;

    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    setup_work(&config, shutdown_rx, Arc::clone(&store_map)).await;

    info!("Saving store...");
    teardown(store_map).await?;
    info!("Shutting down");

    Ok(())
}

async fn setup_store_map(
) -> Result<Arc<HashMap<League, Arc<RwLock<store::Store>>>>, Box<dyn std::error::Error>> {
    let store = Arc::new(RwLock::new(store::load_store(League::Challenge).await?));
    let store_hc = Arc::new(RwLock::new(
        store::load_store(League::ChallengeHardcore).await?,
    ));

    let mut store_map = HashMap::new();
    store_map.insert(League::Challenge, store);
    store_map.insert(League::ChallengeHardcore, store_hc);
    let store_map = Arc::new(store_map);

    Ok(store_map)
}

async fn teardown(store_map: Arc<StoreMap>) -> Result<(), Box<dyn std::error::Error>> {
    store_map
        .get(&League::Challenge)
        .unwrap()
        .read()
        .await
        .persist()?;
    store_map
        .get(&League::ChallengeHardcore)
        .unwrap()
        .read()
        .await
        .persist()?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

fn setup_tracing() -> Result<(), opentelemetry::trace::TraceError> {
    info!("Setup tracing...");
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("trade")
        .with_auto_split_batch(true)
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

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

        info!("Shutting down gracefully");
        break;
    });
}

async fn setup_work(config: &Config, mut shutdown_rx: Receiver<()>, store_map: Arc<StoreMap>) {
    let (api_metrics, store_metrics, store_metrics_hc) =
        setup_metrics(config).expect("failed to setup metrics");

    tokio::select! {
        _ = setup_league(config, Arc::clone(&store_map), store_metrics, League::Challenge) => {},
        _ = setup_league(config, Arc::clone(&store_map), store_metrics_hc, League::ChallengeHardcore) => {},
        _ = async {
            loop {
                if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) = shutdown_rx.try_recv() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        } => {},
        _ = api::init(([0, 0, 0, 0], 4001), Arc::clone(&store_map), api_metrics) => {},
    };
}

async fn setup_league(
    config: &Config,
    store_map: Arc<StoreMap>,
    metrics: impl StoreMetrics + Clone + Send + Sync + std::fmt::Debug + 'static,
    league: League,
) {
    match consumer::setup_rabbitmq_consumer(config, store_map, metrics, league).await {
        Err(e) => error!("Error setting up RabbitMQ consumer: {:?}", e),
        Ok(_) => info!("Consumer decomissioned"),
    }
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
