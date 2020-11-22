mod config;
mod filter;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::filter::{filter_items_from_stash, Filter, Item};
use crate::persistence::Persist;
use crate::schema::stash_records;
use chrono::prelude::*;
use dotenv::dotenv;
use river_subscription::{Indexer, IndexerMessage};
use serde::Serialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let config =
        config::Settings::new().expect("Your configuration file is malformed. Please check.");

    let database_url = std::env::var("DATABASE_URL").expect("No database url set");
    let persistence = persistence::PgDb::new(&database_url);

    let indexer = Indexer::new();
    let rx = indexer.start_with_latest()?;

    let filters: Vec<Filter> = vec![Box::new(|item: &Item| {
        item.extended.category.eq(&"currency")
    })];

    while let Ok(msg) = rx.recv() {
        log::info!("Found {} stash tabs", msg.payload.stashes.len());
        let stashes = map_to_stash_records(msg)
            .into_iter()
            .map(|stash| {
                let (filtered_stash, _n_total, _n_filtered) =
                    filter_items_from_stash(stash, &filters);
                // log::debug!("Filtered {} from {} items from stash", n_filtered, n_total);
                filtered_stash
            })
            // Skip stash records without any items
            .filter(|stash_record| !stash_record.items.as_array().unwrap().is_empty())
            .collect::<Vec<_>>();
        persistence.save(&stashes).expect("Persisting failed");
    }

    Ok(())
}

#[derive(Serialize, Insertable)]
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
}

fn map_to_stash_records(msg: IndexerMessage) -> Vec<StashRecord> {
    let IndexerMessage {
        change_id,
        created_at,
        payload,
    } = msg;
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
        })
        .collect::<Vec<_>>()
}
