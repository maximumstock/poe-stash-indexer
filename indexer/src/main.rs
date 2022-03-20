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

use crate::{
    config::{user_config::RestartMode, Configuration},
    resumption::State,
    sinks::postgres::Postgres,
};
use crate::{filter::filter_stash_record, sinks::rabbitmq::RabbitMq};
use crate::{metrics::setup_metrics, sinks::sink::*};
use crate::{resumption::StateWrapper, stash_record::map_to_stash_records};

use dotenv::dotenv;
use stash_api::{common::ChangeId, sync::Indexer, sync::IndexerMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let config = Configuration::from_env()?;
    log::info!("Chosen configuration: {:#?}", config);

    let signal_flag = setup_signal_handlers()?;
    let metrics = setup_metrics(config.metrics_port)?;
    let sinks = setup_sinks(&config)?;

    let mut resumption = StateWrapper::load_from_file(&"./indexer_state.json");
    let mut indexer = Indexer::new();
    let rx = match (&config.user_config.restart_mode, &resumption.inner) {
        (RestartMode::Fresh, _) => indexer.start_with_latest(),
        (RestartMode::Resume, Some(next)) => {
            indexer.start_with_id(ChangeId::from_str(&next.next_change_id).unwrap())
        }
        (RestartMode::Resume, None) => {
            log::info!("No previous data found, falling back to RestartMode::Fresh");
            indexer.start_with_latest()
        }
    };

    let mut next_chunk_id = resumption.chunk_counter();

    while let Ok(msg) = rx.recv() {
        if signal_flag.load(Ordering::Relaxed) && !indexer.is_stopping() {
            log::info!("Shutdown signal detected. Shutting down gracefully.");
            indexer.stop();
        }

        match msg {
            IndexerMessage::Stop => break,
            IndexerMessage::RateLimited(timer) => {
                log::info!("Rate limited for {} seconds...waiting", timer.as_secs());
            }
            IndexerMessage::Tick {
                change_id,
                payload,
                created_at,
            } => {
                log::info!(
                    "Processing {} ({} stashes)",
                    change_id,
                    payload.stashes.len()
                );

                metrics
                    .stashes_processed
                    .inc_by(payload.stashes.len().try_into().unwrap());
                metrics.chunks_processed.inc();

                let next_change_id = payload.next_change_id.clone();
                let stashes =
                    map_to_stash_records(change_id.clone(), created_at, payload, next_chunk_id)
                        .filter_map(|mut stash| match filter_stash_record(&mut stash, &config) {
                            filter::FilterResult::Block { reason } => {
                                log::debug!("Filter: Blocked stash, reason: {}", reason);
                                None
                            }
                            filter::FilterResult::Pass => Some(stash),
                            filter::FilterResult::Filter {
                                n_total,
                                n_retained,
                            } => {
                                let n_removed = n_total - n_retained;
                                if n_removed > 0 {
                                    log::debug!(
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
        log::info!("Saved resumption state");
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
        log::info!("Configured RabbitMQ fanout sink");
    }

    if let Some(url) = &config.database_url {
        if !url.is_empty() {
            sinks.push(Box::new(Postgres::connect(url)));
            log::info!("Configured PostgreSQL sink");
        }
    }

    Ok(sinks)
}
