extern crate dotenv;

mod config;
mod differ;
mod s3;
mod store;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{config::Configuration, s3::S3Sink};
use stash_api::{
    common::poe_ninja_client::PoeNinjaClient,
    r#async::indexer::{Indexer, IndexerMessage},
};
use tracing::info;
use trade_common::telemetry::setup_telemetry;

use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    setup_telemetry("differ").expect("Telemetry setup");
    let signal_flag = setup_signal_handlers()?;

    let config = Configuration::from_env()?;
    tracing::info!("Chosen configuration: {:#?}", config);

    let mut sink = match config.s3 {
        Some(c) => S3Sink::connect(c.bucket_name, c.access_key, c.secret_key, c.region).await,
        None => anyhow::bail!("no"),
    };

    let client_id = config.client_id.clone();
    let client_secret = config.client_secret.clone();
    let developer_mail = config.developer_mail.clone();

    let indexer = Indexer::new(client_id, client_secret, developer_mail);

    let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async()
        .await
        .unwrap();
    let mut rx = indexer.start_at_change_id(latest_change_id).await;

    let mut store = store::StashStore::new();

    while let Some(msg) = rx.recv().await {
        if signal_flag.load(Ordering::Relaxed) {
            tracing::info!("Shutdown signal detected. Shutting down gracefully.");
            break;
        }

        match msg {
            IndexerMessage::Stop => break,
            IndexerMessage::RateLimited(timer) => {
                tracing::info!("Rate limited for {} seconds...waiting", timer.as_secs());
            }
            IndexerMessage::Tick {
                change_id, stashes, ..
            } => {
                let n_stashes = stashes.len();
                let events = store.ingest(stashes, change_id.to_string());
                tracing::info!(
                    "Chunk ID: {} - {} events from {} stashes",
                    change_id,
                    events.len(),
                    n_stashes
                );
                sink.handle(events).await;
            }
        }
    }

    info!("Flushing sinks");
    sink.flush().await?;

    Ok(())
}

fn setup_signal_handlers() -> anyhow::Result<Arc<AtomicBool>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
