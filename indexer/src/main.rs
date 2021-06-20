mod config;
mod filter;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use crate::config::{Configuration, RestartMode};
use crate::filter::filter_stash_record;
use crate::persistence::Persist;
use crate::schema::stash_records;
use chrono::prelude::*;
use dotenv::dotenv;
use river_subscription::{ChangeId, Indexer, IndexerMessage, StashTabResponse};
use serde::Serialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let config =
        Configuration::read().expect("Your configuration file is malformed. Please check.");

    log::debug!("{:#?}", config);

    let database_url = std::env::var("DATABASE_URL").expect("No database url set");
    let persistence = persistence::PgDb::new(&database_url);

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

    while let Ok(msg) = rx.recv() {
        if signal_flag.load(Ordering::Relaxed) && !indexer.is_stopping() {
            log::info!("CTRL+C detected -> shutting down...");
            indexer.stop();
        }

        match msg {
            IndexerMessage::Stop => break,
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
                    .into_iter()
                    .filter_map(|mut stash| match filter_stash_record(&mut stash, &config) {
                        filter::FilterResult::Block { reason, .. } => {
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
                    // Skip stash records without any items
                    .filter(|stash_record| !stash_record.items.as_array().unwrap().is_empty())
                    .collect::<Vec<_>>();
                persistence.save(&stashes).expect("Persisting failed");
            }
        }

        next_chunk_id += 1;
    }

    log::info!("Shutting down indexer...");

    Ok(())
}

#[derive(Serialize, Insertable, Queryable)]
#[table_name = "stash_records"]
pub struct StashRecord {
    created_at: NaiveDateTime,
    change_id: String,
    next_change_id: String,
    stash_id: String,
    stash_type: String,
    items: serde_json::Value,
    public: bool,
    account_name: Option<String>,
    last_character_name: Option<String>,
    stash_name: Option<String>,
    league: Option<String>,
    chunk_id: i64,
}

fn map_to_stash_records(
    change_id: ChangeId,
    created_at: SystemTime,
    payload: StashTabResponse,
    chunk_id: i64,
) -> Vec<StashRecord> {
    let next_change_id = payload.next_change_id;

    payload
        .stashes
        .into_iter()
        // Ignore stash tabs flagged as private, whose updates are always empty
        .filter(|stash| stash.public)
        .map(move |stash| StashRecord {
            account_name: stash.account_name,
            last_character_name: stash.last_character_name,
            stash_id: stash.id,
            stash_name: stash.stash,
            stash_type: stash.stash_type,
            items: serde_json::to_value(stash.items).expect("Serialization failed"),
            public: stash.public,
            league: stash.league,
            change_id: change_id.clone().into(),
            created_at: DateTime::<Utc>::from(created_at).naive_utc(),
            next_change_id: next_change_id.clone(),
            chunk_id,
        })
        .collect::<Vec<_>>()
}
