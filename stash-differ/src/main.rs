use serde::Serialize;
use sqlx::postgres::PgPoolOptions;

use stash_differ::{
    group_stash_records_by_account_name, DiffStats, LeagueStore, StashRecordIterator,
};

fn main() -> Result<(), sqlx::Error> {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // fishtank Ultimatum
    // let fishtank = "postgres://poe:poe@fishtank:5432/poe-ultimatum";
    // let fishtank_change_id = "1126516147-1133722327-1092891193-1225428360-1174941189";
    // let fishtank_league = "Ultimatum";

    // fishtank Ritual
    let fishtank = "postgres://poe:poe@fishtank:5432/poe";
    let fishtank_league = "Ritual";

    let db_url = fishtank;
    // let initial_change_id = fishtank_change_id;
    let league = fishtank_league;

    let pool = runtime.block_on(async {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
    })?;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats.csv")
        .unwrap();

    let mut store = LeagueStore::new();
    let mut tick = 0;

    let mut iterator = StashRecordIterator::new(&pool, &runtime, 10000, league);

    while let Some(chunk) = iterator.next_chunk() {
        // println!("Chunk length {}", chunk.len());
        let grouped_stashes = group_stash_records_by_account_name(&chunk);

        if tick % 50 == 0 {
            println!(
                "Processing {} accounts in page #{} - last timestamp: {}",
                grouped_stashes.len(),
                tick,
                chunk.last().unwrap().created_at
            );
        }

        // Collect DiffEvents for each account and create Records from them
        grouped_stashes
            .iter()
            .flat_map(|(account_name, stash_records)| {
                let stash = stash_records.as_slice().into();
                if let Some(events) = store.ingest_account(account_name, stash) {
                    let diff_stats: DiffStats = events.as_slice().into();
                    Some(CsvRecord {
                        account_name,
                        tick,
                        n_added: diff_stats.added,
                        n_removed: diff_stats.removed,
                        n_note_changed: diff_stats.note,
                        n_stack_size_changed: diff_stats.stack_size,
                    })
                } else {
                    None
                }
            })
            .for_each(|r| {
                csv_writer
                    .serialize(&r)
                    .unwrap_or_else(|_| panic!("Error when serializing record {:?}", r));
            });

        tick += 1;
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct CsvRecord<'a> {
    account_name: &'a String,
    tick: usize,
    n_added: u32,
    n_removed: u32,
    n_note_changed: u32,
    n_stack_size_changed: u32,
}
