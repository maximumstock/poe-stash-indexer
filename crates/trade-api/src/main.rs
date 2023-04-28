mod api;
mod config;
mod metrics;
mod store;

use config::Config;
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::oneshot::{Receiver, Sender};
use trade_common::{
    assets::AssetIndex,
    telemetry::{setup_telemetry, teardown_telemetry},
};

use tracing::info;

use crate::{metrics::setup_metrics, store::Store};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;
    info!("Configuration {:#?}", config);

    setup_telemetry("trade-api").expect("Telemtry setup");

    let pool = PgPool::connect(&config.db_url).await?;
    let mut index = AssetIndex::new();
    index.init().await?;
    let store = Arc::new(Store::new(index, pool));

    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);
    setup_work(&config, shutdown_rx, store).await;
    teardown_telemetry();

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

async fn setup_work(config: &Config, shutdown_rx: Receiver<()>, store: Arc<Store>) {
    let api_metrics = setup_metrics(config).unwrap();

    tokio::select! {
        _ = async { let _ = shutdown_rx.await; } => {},
        _ = api::init(([0, 0, 0, 0], 4001), api_metrics, store) => {},
    };
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
