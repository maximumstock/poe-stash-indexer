mod config;
mod filter;
mod metrics;
mod resumption;
mod schema;
mod sinks;
mod stash_record;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::{
    convert::TryInto,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::metrics::setup_metrics;
use crate::{
    config::{user_config::RestartMode, Configuration},
    resumption::State,
    sinks::postgres::Postgres,
};
use crate::{filter::filter_stash_record, sinks::rabbitmq::RabbitMq};
use crate::{resumption::StateWrapper, stash_record::map_to_stash_records};
use futures::StreamExt;

use dotenv::dotenv;
use sinks::sink::Sink;
use stash_api::{
    common::{poe_ninja_client::PoeNinjaClient, ChangeId},
    r#async::indexer::{Config, Indexer, IndexerMessage},
};
use trade_common::telemetry::setup_telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    setup_telemetry("indexer").expect("Telemetry setup");

    let client_id = std::env::var("POE_CLIENT_ID").expect("CLIENT_ID environment variable");
    let client_secret =
        std::env::var("POE_CLIENT_SECRET").expect("CLIENT_SECRET environment variable");
    let indexer_config = Config::new(client_id, client_secret);

    let config = Configuration::from_env()?;
    tracing::info!("Chosen configuration: {:#?}", config);

    let signal_flag = setup_signal_handlers()?;
    let metrics = setup_metrics(config.metrics_port)?;
    let sinks = setup_sinks(&config)?;

    let mut resumption = StateWrapper::load_from_file(&"./indexer_state.json");
    let mut indexer = Indexer::new();
    let mut rx = match (&config.user_config.restart_mode, &resumption.inner) {
        (RestartMode::Fresh, _) => {
            let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
            indexer.start_at_change_id(indexer_config, latest_change_id)
        }
        (RestartMode::Resume, Some(next)) => indexer.start_at_change_id(
            indexer_config,
            ChangeId::from_str(&next.next_change_id).unwrap(),
        ),
        (RestartMode::Resume, None) => {
            tracing::info!("No previous data found, falling back to RestartMode::Fresh");
            let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async().await?;
            indexer.start_at_change_id(indexer_config, latest_change_id)
        }
    }
    .await;

    let mut next_chunk_id = resumption.chunk_counter();

    while let Some(msg) = rx.next().await {
        if signal_flag.load(Ordering::Relaxed) && !indexer.is_stopping() {
            tracing::info!("Shutdown signal detected. Shutting down gracefully.");
            indexer.stop();
        }

        match msg {
            IndexerMessage::Stop => break,
            IndexerMessage::RateLimited(timer) => {
                tracing::info!("Rate limited for {} seconds...waiting", timer.as_secs());
                metrics.rate_limited.inc();
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

                metrics
                    .stashes_processed
                    .inc_by(response.stashes.len().try_into().unwrap());
                metrics.chunks_processed.inc();

                let next_change_id = response.next_change_id.clone();
                let stashes =
                    map_to_stash_records(change_id.clone(), created_at, response, next_chunk_id)
                        .filter_map(|mut stash| match filter_stash_record(&mut stash, &config) {
                            filter::FilterResult::Block { reason } => {
                                tracing::debug!("Filter: Blocked stash, reason: {}", reason);
                                None
                            }
                            filter::FilterResult::Pass => Some(stash),
                            filter::FilterResult::Filter {
                                n_total,
                                n_retained,
                            } => {
                                let n_removed = n_total - n_retained;
                                if n_removed > 0 {
                                    tracing::debug!(
                                        "Filter: Removed {} \t Retained {} \t Total {}",
                                        n_removed,
                                        n_retained,
                                        n_total
                                    );
                                }
                                Some(stash)
                            }
                        })
                        .collect::<Vec<_>>();

                if !stashes.is_empty() {
                    next_chunk_id += 1;
                    for sink in &sinks {
                        sink.handle(&stashes)?;
                    }
                }

                // Update resumption state at the end of each tick
                resumption.update(State {
                    change_id: change_id.to_string(),
                    next_change_id,
                    chunk_counter: next_chunk_id,
                });
            }
        }
    }

    if resumption.save().is_ok() {
        tracing::info!("Saved resumption state");
    }

    if indexer.is_stopping() {
        Ok(())
    } else {
        std::process::exit(-1);
    }
}

fn setup_signal_handlers() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;
    Ok(signal_flag)
}

fn setup_sinks<'a>(
    config: &'a Configuration,
) -> Result<Vec<Box<dyn Sink + 'a>>, Box<dyn std::error::Error>> {
    let mut sinks: Vec<Box<dyn Sink>> = vec![];

    if let Some(conf) = &config.rabbitmq {
        let mq_sink = RabbitMq::connect(conf)?;
        sinks.push(Box::new(mq_sink));
        tracing::info!("Configured RabbitMQ fanout sink");
    }

    if let Some(url) = &config.database_url {
        if !url.is_empty() {
            sinks.push(Box::new(Postgres::connect(url)));
            tracing::info!("Configured PostgreSQL sink");
        }
    }

    Ok(sinks)
}
