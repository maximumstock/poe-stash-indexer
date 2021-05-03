use std::collections::{HashMap, VecDeque};

use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use stash_differ::{DiffStats, LeagueStore, StashRecord};

const PAGE_SIZE: i64 = 5000;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
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

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats.csv")
        .unwrap();

    let mut queue = VecDeque::<(i64, i64)>::new();
    queue.push_back((0, PAGE_SIZE));
    let mut store = LeagueStore::new();
    let mut tick = 0;

    while let Some(page) = queue.pop_front() {
        println!("Fetching next page: {:?}", page);

        let stash_records = fetch_stash_records_paginated(&pool, page.0, page.1, league).await?;
        let grouped_stashes = group_stash_records_by_account_name(&stash_records);

        println!(
            "Processing {} accounts in page #{} - last timestamp: {}",
            grouped_stashes.len(),
            tick,
            stash_records.last().unwrap().created_at
        );

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

        println!("Store size: {:?} elements", store.inner.len());
        queue.push_back((page.1, page.1 + PAGE_SIZE));
        tick += 1;
    }

    Ok(())
}

async fn fetch_stash_records_paginated(
    pool: &Pool<Postgres>,
    start: i64,
    end: i64,
    league: &str,
) -> Result<Vec<StashRecord>, sqlx::Error> {
    sqlx::query_as::<_, StashRecord>(
        "SELECT change_id, next_change_id, stash_id, account_name, league, items, created_at
             FROM stash_records
             WHERE league = $1 and int8range($2, $3, '[]') @> int8range(id, id, '[]')",
    )
    .bind(league)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

fn group_stash_records_by_account_name(
    stash_records: &[StashRecord],
) -> HashMap<String, Vec<StashRecord>> {
    let mut out = HashMap::new();

    for sr in stash_records {
        if let Some(account_name) = &sr.account_name {
            out.entry(account_name.clone())
                .or_insert_with(Vec::new)
                .push(sr.clone())
        }
    }

    out
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
