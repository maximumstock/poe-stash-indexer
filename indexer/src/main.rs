mod config;
mod filter;
mod schema;
mod sinks;
mod stash_record;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::sinks::sink::*;
use crate::sinks::{postgres::Postgres, rabbitmq::RabbitMqConfig};
use crate::{
    config::{Configuration, RestartMode},
    stash_record::map_to_stash_records,
};
use crate::{filter::filter_stash_record, sinks::rabbitmq::RabbitMq};

use dotenv::dotenv;
use stash_api::{common::ChangeId, sync::Indexer, sync::IndexerMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let config =
        Configuration::read().expect("Your configuration file is malformed. Please check.");

    log::info!("Chosen configuration: {:#?}", config);

    let mut sinks: Vec<Box<dyn Sink>> = vec![];

    if let Some(config) = RabbitMqConfig::from_env()? {
        let mq_sink = RabbitMq::connect(config)?;
        sinks.push(Box::new(mq_sink));
        log::info!("Configured RabbitMQ fanout sink");
    }

    let database_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let persistence = Postgres::new(&database_url);
    sinks.push(Box::new(Postgres::new(&database_url)));
    log::info!("Configured PostgreSQL sink");

    let mut indexer = Indexer::new();
    let last_change_id: diesel::result::QueryResult<String> = persistence.get_next_change_id();
    let mut next_chunk_id = persistence
        .get_next_chunk_id()
        .expect("Failed to read last chunk id")
        .map(|id| id + 1)
        .unwrap_or(0);
    let rx = match (&config.restart_mode, last_change_id) {
        (RestartMode::Fresh, _) => indexer.start_with_latest(),
        (RestartMode::Resume, Ok(id)) => indexer.start_with_id(ChangeId::from_str(&id).unwrap()),
        (RestartMode::Resume, Err(_)) => {
            log::info!("No previous data found, falling back to RestartMode::Fresh");
            indexer.start_with_latest()
        }
    };

    let signal_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, signal_flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, signal_flag.clone())?;

    while let Ok(msg) = rx.recv() {
        if signal_flag.load(Ordering::Relaxed) && !indexer.is_stopping() {
            log::info!("CTRL+C detected -> shutting down...");
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
                let stashes = map_to_stash_records(change_id, created_at, payload, next_chunk_id)
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
            }
        }
    }

    log::info!("Shutting down indexer...");

    Ok(())
}
