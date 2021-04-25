use std::collections::{HashMap, VecDeque};

use futures::future::FutureExt;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use stash_differ::{DiffStats, LeagueStore, StashRecord};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // fishtank Ultimatum
    // let fishtank = "postgres://poe:poe@fishtank:5432/poe-ultimatum";
    // let fishtank_change_id = "1126516147-1133722327-1092891193-1225428360-1174941189";
    // let fishtank_league = "Ultimatum";

    // fishtank Ritual
    let fishtank = "postgres://poe:poe@fishtank:5432/poe";
    // first Ritual chunk, created_at = 2021-01-15
    let fishtank_change_id = "933167925-944185125-905522013-1020175612-28136316";
    let fishtank_league = "Ritual";

    // // localhost
    // let localhost = "postgres://poe:poe@localhost:5432/poe";
    // let localhost_change_id = "874427962-886797677-848596200-956890878-915966255";
    // let localhost_league = "Ritual";

    let db_url = fishtank;
    let initial_change_id = fishtank_change_id;
    let league = fishtank_league;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("diff_stats.csv")
        .unwrap();

    let mut queue = VecDeque::<String>::new();
    queue.push_back(initial_change_id.into());
    let mut store = LeagueStore::new();
    let mut tick = 0;

    while let Some(change_id) = queue.pop_front() {
        let next_change_id = fetch_next_change_id(&pool, &change_id).await?;
        let stash_records = fetch_stash_records(&pool, &change_id, &league).await?;

        if stash_records.is_empty() {
            println!(
                "No data for {:?} -> Jumping to successor {:?}",
                change_id, next_change_id
            );
            queue.push_back(next_change_id);
            continue;
        }

        let account_stash_data = group_stash_records_by_account_name(&stash_records);

        println!(
            "Processing {} accounts in chunk #{}",
            account_stash_data.len(),
            tick
        );

        // Collect DiffEvents for each account and create Records from them
        account_stash_data
            .iter()
            .flat_map(|(account_name, stash_records)| {
                if let Some(events) = store.ingest_account(account_name, stash_records) {
                    // println!(
                    //     "\t\t{:?} diff events for account {:?}",
                    //     events.len(),
                    //     account_name
                    // );
                    let diff_stats: DiffStats = events.as_slice().into();
                    Some(Record {
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
        let next_change_id = stash_records.get(0).unwrap().next_change_id.clone();
        queue.push_back(next_change_id);
        tick += 1;
    }

    Ok(())
}

async fn fetch_stash_records(
    pool: &Pool<Postgres>,
    change_id: &str,
    league: &str,
) -> Result<Vec<StashRecord>, sqlx::Error> {
    sqlx::query_as::< _, StashRecord>(
        "SELECT change_id, next_change_id, stash_id, account_name, league, items FROM stash_records WHERE change_id = $1 and league = $2"
    )
    .bind(change_id)
    .bind(league)
    .fetch_all(pool).await
}

async fn fetch_next_change_id(
    pool: &Pool<Postgres>,
    change_id: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_as::< _, StashRecord>(
        "SELECT change_id, next_change_id, stash_id, account_name, league, items FROM stash_records WHERE change_id = $1"
    )
    .bind(change_id)
    .fetch_one(pool)
    .map(|x| x.map(|sr| sr.next_change_id))
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
struct Record<'a> {
    account_name: &'a String,
    tick: usize,
    n_added: u32,
    n_removed: u32,
    n_note_changed: u32,
    n_stack_size_changed: u32,
}
