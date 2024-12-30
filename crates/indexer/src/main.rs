mod config;
mod metrics;
mod resumption;
mod sinks;

extern crate dotenv;

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::resumption::State;
use crate::resumption::StateWrapper;
use crate::sinks::rabbitmq::RabbitMqSink;
use crate::{metrics::setup_metrics, sinks::s3::S3Sink};

use config::{Configuration, RestartMode};
use sinks::{postgres::PostgresSink, sink::Sink};
use stash_api::{
    common::{poe_ninja_client::PoeNinjaClient, ChangeId},
    r#async::indexer::{Indexer, IndexerMessage},
};
use tracing::info;
use trade_common::{assets::AssetIndex, telemetry::setup_telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    setup_telemetry("indexer").expect("Telemetry setup");

    let config = Configuration::from_env()?;
    tracing::info!("Chosen configuration: {:#?}", config);

    let signal_flag = setup_signal_handlers()?;
    let metrics = setup_metrics(config.metrics_port)?;
    let mut sinks = setup_sinks(config.clone()).await?;

    let mut resumption = StateWrapper::load_from_file(&"./indexer_state.json");
    let indexer = Indexer::new(
        config.client_id.clone(),
        config.client_secret.clone(),
        config.developer_mail.clone(),
    );
    let mut rx = match (&config.restart_mode, &resumption.inner) {
        (RestartMode::Fresh, _) => {
            let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
            indexer.start_at_change_id(latest_change_id)
        }
        (RestartMode::Resume, Some(next)) => {
            indexer.start_at_change_id(ChangeId::from_str(&next.next_change_id).unwrap())
        }
        (RestartMode::Resume, None) => {
            tracing::info!("No previous data found, falling back to RestartMode::Fresh");
            let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
            indexer.start_at_change_id(latest_change_id)
        }
    }
    .await;

    while let Some(msg) = rx.recv().await {
        if signal_flag.load(Ordering::Relaxed) {
            tracing::info!(
                "Shutdown signal detected. Shutting down gracefully while flushing sinks."
            );
            break;
        }

        match msg {
            IndexerMessage::Stop => break,
            IndexerMessage::RateLimited(timer) => {
                tracing::info!("Rate limited for {} seconds...waiting", timer.as_secs());
                metrics.rate_limited.inc();
            }
            IndexerMessage::Tick {
                change_id,
                stashes,
                next_change_id,
                ..
            } => {
                tracing::info!("Processing {} ({} stashes)", change_id, stashes.len());

                metrics
                    .stashes_processed
                    .inc_by(stashes.len().try_into().unwrap());
                metrics.chunks_processed.inc();

                let next_change_id = next_change_id.clone();

                if !stashes.is_empty() {
                    for sink in sinks.iter_mut() {
                        sink.handle(&stashes).await?;
                    }
                }

                // Update resumption state at the end of each tick
                resumption.update(State {
                    change_id: change_id.to_string(),
                    next_change_id: next_change_id.to_string(),
                });
            }
        }
    }

    if !sinks.is_empty() {
        info!("Flushing sinks");
        for sink in sinks.iter_mut() {
            sink.flush().await?;
        }
    }

    match resumption.save() {
        Ok(_) => tracing::info!("Saved resumption state"),
        Err(_) => tracing::error!("Saving resumption state failed"),
    }

    Ok(())
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}

async fn setup_sinks<'a>(
    config: Configuration,
) -> Result<Vec<Box<dyn Sink>>, Box<dyn std::error::Error>> {
    let mut sinks: Vec<Box<dyn Sink>> = vec![];

    if let Some(conf) = config.rabbitmq {
        let mq_sink = RabbitMqSink::connect(conf).await?;
        sinks.push(Box::new(mq_sink));
        tracing::info!("Configured RabbitMQ fanout sink");
    }

    if let Some(config) = config.s3 {
        let s3_sink = S3Sink::connect(&config.bucket_name, &config.region).await?;
        sinks.push(Box::new(s3_sink));
        tracing::info!("Configured S3 sink");
    }

    if let Some(config) = config.postgres {
        let mut asset_index = AssetIndex::new();
        asset_index.init().await?;

        let postgres_sink = PostgresSink::connect(&config, asset_index).await.unwrap();
        sinks.push(Box::new(postgres_sink));
        tracing::info!("Configured PostgreSQL sink");
    }

    Ok(sinks)
}
