use serde::Serialize;

use log::info;

use crate::{
    differ::DiffEvent, stash::group_stash_records_by_account_name, store::LeagueStore, SharedState,
};

/// A consumer thread implementation that pulls data from `SharedState`, builds a
/// continuous stash store, build diffs events and serializes them to a .csv file.
pub fn flat_consumer(shared_state: SharedState) {
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats_flat.csv")
        .unwrap();

    let mut store = LeagueStore::new();
    let mut page_idx = 0;

    let (state, cvar) = &*shared_state;

    let mut lock = state.lock().unwrap();

    loop {
        lock = cvar.wait_while(lock, |s| s.queue.is_empty()).unwrap();

        if let Some(chunk) = lock.queue.pop_front() {
            let chunk_first_timestamp = chunk.first().unwrap().created_at;

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
                .map(|(account_name, stash_records)| {
                    let stash = stash_records.into();
                    let diff_events = store.diff_account(&account_name, &stash);
                    store.update_account(account_name.as_str(), stash);
                    (account_name, diff_events)
                })
                .for_each(|(account_name, events)| {
                    if let Some(events) = events {
                        events.iter().for_each(|event| {
                            let record = CsvRecord {
                                account_name: &account_name,
                                last_timestamp: chunk_first_timestamp.timestamp(),
                                event,
                            };
                            csv_writer.serialize(&record).unwrap_or_else(|e| {
                                log::error!("{}", e);
                                panic!("Error when serializing record {:?}", record)
                            });
                        });
                    }
                });
        }

        page_idx += 1;
    }
}

#[derive(Debug, Serialize)]
struct CsvRecord<'a> {
    account_name: &'a String,
    last_timestamp: i64,
    #[serde(flatten)]
    event: &'a DiffEvent,
}
