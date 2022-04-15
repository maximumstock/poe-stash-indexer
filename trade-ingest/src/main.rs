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
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{
    oneshot::{Receiver, Sender},
};

use tracing::{error, info};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

use crate::{assets::AssetIndex, metrics::setup_metrics};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.db_url)
            .await?,
    );

    sqlx::migrate!("./migrations").run(&*pool).await?;

    setup_tracing().expect("Tracing setup failed");

    let signal_flag = setup_signal_handlers()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    setup_shutdown_handler(signal_flag, shutdown_tx);

    setup_work(&config, pool, shutdown_rx).await;

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

async fn setup_work(config: &Config, pool: Arc<Pool<Postgres>>, mut shutdown_rx: Receiver<()>) {
    let (_api_metrics, store_metrics, store_metrics_hc) =
        setup_metrics(config).expect("failed to setup metrics");

    let mut asset_index = AssetIndex::new();
    asset_index.init().await.unwrap();
    let asset_index = Arc::new(asset_index);

    tokio::select! {
        _ = setup_league(config, Arc::clone(&pool), store_metrics, Arc::clone(&asset_index), League::Challenge) => {},
        _ = setup_league(config, pool,  store_metrics_hc,Arc::clone(&asset_index),League::ChallengeHardcore) => {},
        _ = async {
            loop {
                if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) = shutdown_rx.try_recv() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        } => {},
    };
}

async fn setup_league(
    config: &Config,
    pool: Arc<Pool<Postgres>>,
    metrics: impl StoreMetrics + Clone + Send + Sync + std::fmt::Debug + 'static,
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
