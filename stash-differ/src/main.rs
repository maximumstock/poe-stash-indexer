use std::sync::mpsc::{self, Receiver, SyncSender};

use serde::Serialize;
use sqlx::postgres::PgPoolOptions;

use stash_differ::{
    group_stash_records_by_account_name, DiffStats, LeagueStore, StashRecord, StashRecordIterator,
};

fn main() -> Result<(), sqlx::Error> {
    let (tx, rx) = mpsc::sync_channel::<Vec<StashRecord>>(5);
    let producer = std::thread::spawn(|| producer(tx));
    let consumer = std::thread::spawn(|| consumer(rx));

    producer.join().unwrap();
    consumer.join().unwrap();

    Ok(())
}

fn consumer(rx: Receiver<Vec<StashRecord>>) {
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats.csv")
        .unwrap();
    let mut store = LeagueStore::new();
    let mut tick = 0;

    while let Ok(chunk) = rx.recv() {
        // println!("Chunk length {}", chunk.len());
        let grouped_stashes = group_stash_records_by_account_name(&chunk);

        if tick % 500 == 0 {
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
                let diff_stats = store.diff_account(account_name, &stash).map(|events| {
                    let stats: DiffStats = events.as_slice().into();

                    CsvRecord {
                        account_name,
                        tick,
                        n_added: stats.added,
                        n_removed: stats.removed,
                        n_note_changed: stats.note,
                        n_stack_size_changed: stats.stack_size,
                    }
                });
                store.update_account(account_name, stash);
                diff_stats
            })
            .for_each(|r| {
                csv_writer
                    .serialize(&r)
                    .unwrap_or_else(|_| panic!("Error when serializing record {:?}", r));
            });

        tick += 1;
    }
}

fn producer(tx: SyncSender<Vec<StashRecord>>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let fishtank = "postgres://poe:poe@fishtank:5432/poe";
    let fishtank_league = "Ritual";
    let db_url = fishtank;
    let league = fishtank_league;
    let pool = runtime
        .block_on(async {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(db_url)
                .await
        })
        .unwrap();

    let mut iterator = StashRecordIterator::new(&pool, &runtime, 1000, league);

    while let Some(next) = iterator.next_chunk() {
        tx.send(next).expect("sending failed");
    }
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
