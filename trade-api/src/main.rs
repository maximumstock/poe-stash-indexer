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
use trade_common::assets::AssetIndex;

use tracing::info;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

use crate::{metrics::setup_metrics, store::Store};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;

    std::env::set_var("DATABASE_URL", &config.db_url);

    let pool = Arc::new(PgPool::connect(&config.db_url).await?);
    let mut index = AssetIndex::new();
    index.init().await?;
    let store = Store::new(index, pool);

    setup_tracing().expect("Tracing setup failed");

    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    setup_work(&config, shutdown_rx, store).await;

    info!("Saving store...");
    teardown().await?;
    info!("Shutting down");

    Ok(())
}

async fn teardown() -> Result<(), Box<dyn std::error::Error>> {
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

async fn setup_work(config: &Config, mut shutdown_rx: Receiver<()>, store: Store) {
    let (api_metrics,) = setup_metrics(config).expect("failed to setup metrics");

    tokio::select! {
        _ = async {
            loop {
                if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) = shutdown_rx.try_recv() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        } => {},
        _ = api::init(([0, 0, 0, 0], 4001), api_metrics, Arc::new(store)) => {},
    };
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
