extern crate dotenv;

mod config;
mod differ;
mod s3;
mod stash;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{config::Configuration, s3::S3Sink};
use stash_api::{
    common::poe_ninja_client::PoeNinjaClient,
    r#async::indexer::{Indexer, IndexerMessage},
};
use trade_common::telemetry::setup_telemetry;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    setup_telemetry("differ").expect("Telemetry setup");

    let config = Configuration::from_env()?;
    tracing::info!("Chosen configuration: {:#?}", config);

    // let sink = config
    //     .s3
    //     .map(|c| S3Sink::connect(c.bucket_name, c.access_key, c.secret_key, c.region));

    let s = match config.s3 {
        Some(s) => S3Sink::connect(s.bucket_name, s.access_key, s.secret_key, s.region).await,
        None => anyhow::bail!("no"),
    };

    // if let Some(config) = &config.s3 {
    //     let s3_sink = S3Sink::connect(
    //         &config.bucket_name,
    //         &config.access_key,
    //         config.secret_key.clone(),
    //         &config.region,
    //     )
    //     .await?;
    //     sinks.push(Box::new(s3_sink));
    // }

    let signal_flag = setup_signal_handlers()?;

    let client_id = config.client_id.clone();
    let client_secret = config.client_secret.clone();
    let developer_mail = config.developer_mail.clone();

    // let mut resumption = StateWrapper::load_from_file(&"./indexer_state.json");
    let indexer = Indexer::new();
    // let mut rx = match (&config.user_config.restart_mode, &resumption.inner) {
    //     (RestartMode::Fresh, _) => {
    //         let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
    //         indexer.start_at_change_id(client_id, client_secret, developer_mail, latest_change_id)
    //     }
    //     (RestartMode::Resume, Some(next)) => indexer.start_at_change_id(
    //         client_id,
    //         client_secret,
    //         developer_mail,
    //         ChangeId::from_str(&next.next_change_id).unwrap(),
    //     ),
    //     (RestartMode::Resume, None) => {
    //         tracing::info!("No previous data found, falling back to RestartMode::Fresh");
    //         let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
    //         indexer.start_at_change_id(client_id, client_secret, developer_mail, latest_change_id)
    //     }
    // }
    // .await;

    let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async()
        .await
        .unwrap();
    let mut rx = indexer
        .start_at_change_id(client_id, client_secret, developer_mail, latest_change_id)
        .await;

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
                change_id,
                response,
                created_at,
                ..
            } => {
                tracing::info!(
                    "Processing {} ({} stashes)",
                    change_id,
                    response.stashes.len()
                );

                let next_change_id = response.next_change_id.clone();
                // if !stashes.is_empty() {
                //     for sink in sinks.iter_mut() {
                //         sink.handle(&stashes).await?;
                //     }
                // }

                // // Update resumption state at the end of each tick
                // resumption.update(State {
                //     change_id: change_id.to_string(),
                //     next_change_id,
                // });
            }
        }
    }

    // info!("Flushing sinks");
    // for sink in sinks.iter_mut() {
    //     sink.flush().await?;
    // }

    // match resumption.save() {
    //     Ok(_) => tracing::info!("Saved resumption state"),
    //     Err(_) => tracing::error!("Saving resumption state failed"),
    // }

    Ok(())
}

fn setup_signal_handlers() -> anyhow::Result<Arc<AtomicBool>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}
