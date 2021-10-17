mod config;
mod filter;
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

use crate::stash_record::map_to_stash_records;
use crate::{
    config::{user_config::RestartMode, Configuration, RabbitMqConfig},
    sinks::postgres::Postgres,
};
use crate::{filter::filter_stash_record, sinks::rabbitmq::RabbitMq};
use crate::{metrics::init_prometheus_exporter, sinks::sink::*};

use dotenv::dotenv;
use stash_api::{common::ChangeId, sync::Indexer, sync::IndexerMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let metrics_port = std::env::var("METRICS_PORT")
        .expect("Missing METRICS_PORT")
        .parse()
        .unwrap();
    let metrics = init_prometheus_exporter(metrics_port)?;

    let config = Configuration::from_env()?;

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
    let last_change_id = persistence.get_next_change_id();
    let mut next_chunk_id = persistence
        .get_next_chunk_id()
        .expect("Failed to read last chunk id")
        .map(|id| id + 1)
        .unwrap_or(0);
    let rx = match (&config.user_config.restart_mode, last_change_id) {
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

                metrics
                    .stashes_processed
                    .inc_by(payload.stashes.len().try_into().unwrap());
                metrics.chunks_processed.inc();

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

mod metrics {
    use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};

    pub struct Metrics {
        pub chunks_processed: GenericCounter<AtomicU64>,
        pub stashes_processed: GenericCounter<AtomicU64>,
    }

    pub fn init_prometheus_exporter(port: u32) -> Result<Metrics, Box<dyn std::error::Error>> {
        let binding = format!("0.0.0.0:{}", port).parse()?;
        prometheus_exporter::start(binding)?;
        let chunks_processed =
            prometheus_exporter::prometheus::register_int_counter!("chunks_processed", "help")?;

        let stashes_processed =
            prometheus_exporter::prometheus::register_int_counter!("stashes_processed", "help")?;

        Ok(Metrics {
            chunks_processed,
            stashes_processed,
        })
    }
}
