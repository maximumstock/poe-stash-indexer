mod config;
mod consumer;
mod metrics;
mod note_parser;
mod source;
mod store;

use config::Config;
use metrics::store::StoreMetrics;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::oneshot::{Receiver, Sender};
use trade_common::{
    assets::AssetIndex,
    league::League,
    telemetry::{setup_telemetry, teardown_telemetry},
};

use tracing::{error, info};

use crate::metrics::setup_metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_telemetry("trade-ingest").expect("Telemtry setup");

    let config = config::Config::from_env()?;
    info!("Configuration {:#?}", config);

    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    setup_work(&config, shutdown_rx).await?;

    info!("Tearing down...");
    teardown_telemetry();
    info!("Shutting down");

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

async fn setup_work(
    config: &Config,
    mut shutdown_rx: Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.db_url)
            .await?,
    );

    sqlx::migrate!("./migrations").run(&*pool).await?;

    let (store_metrics, store_metrics_hc) = setup_metrics(config).expect("failed to setup metrics");

    let mut asset_index = AssetIndex::new();
    asset_index.init().await.unwrap();
    let asset_index = Arc::new(asset_index);

    tokio::select! {
        _ = setup_league(config, Arc::clone(&pool), store_metrics, Arc::clone(&asset_index), League::Challenge) => {},
        _ = setup_league(config, Arc::clone(&pool), store_metrics_hc, Arc::clone(&asset_index), League::ChallengeHardcore) => {},
        _ = async {
            loop {
                if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) = shutdown_rx.try_recv() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        } => {},
    };

    Ok(())
}

async fn setup_league(
    config: &Config,
    pool: Arc<Pool<Postgres>>,
    metrics: impl StoreMetrics + Send + Sync + 'static,
    asset_index: Arc<AssetIndex>,
    league: League,
) {
    match consumer::setup_rabbitmq_consumer(config, pool, metrics, asset_index, league).await {
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
