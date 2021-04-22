use std::collections::{HashMap, VecDeque};

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use stash_differ::{LeagueStore, StashRecord};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let fishtank = "postgres://poe:poe@fishtank:5432/poe-ultimatum";
    let localhost = "postgres://poe:poe@localhost:5432/poe";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(fishtank)
        .await?;

    // localhost
    let localhost_change_id = "874427962-886797677-848596200-956890878-915966255";
    // fishtank
    let fishtank_change_id = "1126516147-1133722327-1092891193-1225428360-1174941189";
    let mut queue = VecDeque::<String>::new();
    queue.push_back(fishtank_change_id.into());
    let mut store = LeagueStore::new();

    while let Some(change_id) = queue.pop_front() {
        let stash_records = fetch_stash_records_for_change_id(&pool, &change_id).await?;

        if stash_records.is_empty() {
            break;
        }

        let account_stash_data = group_stash_records_by_account_name(&stash_records);

        for (account_name, stash_records) in &account_stash_data {
            if let Some(diff_events) = store.ingest_account(account_name, stash_records) {
                println!(
                    "{:?} diff events for account {:?}",
                    diff_events.len(),
                    account_name
                );
            }
        }

        println!("Store size: {:?} elements", store.inner.len());
        let next_change_id = stash_records.get(0).unwrap().next_change_id.clone();
        queue.push_back(next_change_id);
    }

    Ok(())
}

async fn fetch_stash_records_for_change_id(
    pool: &Pool<Postgres>,
    change_id: &str,
) -> Result<Vec<StashRecord>, sqlx::Error> {
    sqlx::query_as::< _, StashRecord>(
        "SELECT change_id, next_change_id, stash_id, account_name, league, items FROM stash_records WHERE change_id = $1"
    )
    .bind(change_id)
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
