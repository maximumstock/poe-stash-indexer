extern crate log;
extern crate pretty_env_logger;

mod db;
mod differ;
mod stash;
mod store;

use chrono::{prelude::*, Duration};
use db::StashRecordIterator;
use dotenv::dotenv;
use log::info;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use stash::StashRecord;
use std::{
    collections::HashMap,
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::{differ::DiffStats, stash::group_stash_records_by_account_name, store::LeagueStore};

#[derive(Default)]
struct State {
    queue: VecDeque<Vec<StashRecord>>,
}

type SharedState = Arc<(Mutex<State>, Condvar)>;

fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    pretty_env_logger::init();

    let database_url =
        std::env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable");
    let league = std::env::var("LEAGUE").expect("Missing LEAGUE environment variable");

    let shared_state = SharedState::default();
    let shared_state2 = shared_state.clone();

    let producer =
        std::thread::spawn(move || producer(shared_state, database_url.as_ref(), league.as_ref()));

    let consumer = std::thread::spawn(|| consumer(shared_state2));

    producer.join().unwrap();
    consumer.join().unwrap();

    Ok(())
}

fn consumer(shared_state: SharedState) {
    const AGGREGATE_WINDOW: i64 = 60 * 30;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats.csv")
        .unwrap();

    let mut store = LeagueStore::new();
    let mut aggregation_tick = 0;
    let mut diff_stats: HashMap<String, DiffStats> = HashMap::new();
    let mut start_time: Option<NaiveDateTime> = None;
    let mut page_idx = 0;

    let (state, cvar) = &*shared_state;

    let mut lock = state.lock().unwrap();

    loop {
        lock = cvar.wait_while(lock, |s| s.queue.is_empty()).unwrap();

        if let Some(chunk) = lock.queue.pop_front() {
            let chunk_first_timestamp = chunk.first().unwrap().created_at;

            if start_time.is_none() {
                start_time = Some(chunk_first_timestamp + Duration::seconds(AGGREGATE_WINDOW));
            }

            let grouped_stashes = group_stash_records_by_account_name(chunk);

            if page_idx % 5000 == 0 {
                info!(
                    "Processing {} accounts in page #{} - timestamp: {}",
                    grouped_stashes.len(),
                    page_idx,
                    chunk_first_timestamp
                );
            }

            // Collect DiffEvents for each account and create Records from them
            grouped_stashes
                .into_iter()
                .flat_map(|(account_name, stash_records)| {
                    let stash = stash_records.into();
                    let diff_events = store.diff_account(&account_name, &stash);
                    store.update_account(account_name.as_ref(), stash);
                    diff_events.map(|events| {
                        let stats: DiffStats = events.as_slice().into();
                        (account_name, stats)
                    })
                })
                .for_each(|(account_name, ds)| {
                    diff_stats
                        .entry(account_name)
                        .and_modify(|e| *e += ds)
                        .or_insert(ds);
                });

            // Flush accumulated aggregates when a new aggregation window begins
            if start_time
                .map(|s| s < chunk_first_timestamp)
                .unwrap_or(false)
            {
                diff_stats.iter().for_each(|(account_name, stats)| {
                    let record = CsvRecord {
                        account_name,
                        tick: aggregation_tick,
                        last_timestamp: chunk_first_timestamp.timestamp(),
                        n_added: stats.added,
                        n_removed: stats.removed,
                        n_note_changed: stats.note,
                        n_stack_size_changed: stats.stack_size,
                    };
                    csv_writer
                        .serialize(&record)
                        .unwrap_or_else(|_| panic!("Error when serializing record {:?}", record));
                });
                diff_stats.clear();
                start_time = Some(chunk_first_timestamp + Duration::seconds(AGGREGATE_WINDOW));
                aggregation_tick += 1;
            }

            page_idx += 1;
        }
    }
}

fn producer(shared_state: SharedState, database_url: &str, league: &str) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let pool = runtime
        .block_on(async {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
        })
        .unwrap();

    let mut iterator = StashRecordIterator::new(&pool, &runtime, 10000, league);

    while let Some(next) = iterator.next_chunk() {
        let (lock, cvar) = &*shared_state;
        let queue = &mut lock.lock().unwrap().queue;
        queue.push_back(next);

        if queue.len() > 3 {
            cvar.notify_one();
        }
    }
}

#[derive(Serialize, Debug)]
struct CsvRecord<'a> {
    account_name: &'a String,
    last_timestamp: i64,
    tick: usize,
    n_added: u32,
    n_removed: u32,
    n_note_changed: u32,
    n_stack_size_changed: u32,
}
