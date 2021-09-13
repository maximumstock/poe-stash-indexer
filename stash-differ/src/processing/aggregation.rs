use serde::Serialize;
use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime};
use log::info;

use crate::{
    differ::DiffStats, stash::group_stash_records_by_account_name, store::LeagueStore, SharedState,
};

/// A consumer thread implementation that pulls data from `SharedState`, builds a
/// continuous stash store, build diffs and aggregates them based on a time window
/// before serializing these aggregated events to a .csv file.
pub fn aggregation_consumer(shared_state: SharedState) {
    const AGGREGATE_WINDOW: i64 = 60 * 30;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats_aggregated.csv")
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
                    store.update_account(account_name.as_str(), stash);
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

#[derive(Serialize, Debug)]
pub struct CsvRecord<'a> {
    account_name: &'a String,
    last_timestamp: i64,
    tick: usize,
    n_added: u32,
    n_removed: u32,
    n_note_changed: u32,
    n_stack_size_changed: u32,
}
