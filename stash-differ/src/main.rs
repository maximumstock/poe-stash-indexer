use serde::Deserialize;
use sqlx::prelude::*;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Pool, Postgres,
};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://poe:poe@localhost:5432/poe")
        .await?;

    let change_id = "874427962-886797677-848596200-956890878-915966255";
    let rows = fetch_change_id_records(&pool, change_id).await?;

    println!("{:?}", rows);

    Ok(())
}

async fn fetch_change_id_records(
    pool: &Pool<Postgres>,
    change_id: &str,
) -> Result<Vec<StashRecord>, sqlx::Error> {
    sqlx::query_as::< _, StashRecord>(
        "SELECT change_id, stash_id, account_name, league, items FROM stash_records WHERE change_id = $1"
    )
    .bind(change_id)
    .fetch_all(pool)
    .await
}

impl<'r> sqlx::FromRow<'r, PgRow> for StashRecord {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(StashRecord {
            change_id: row.try_get("change_id")?,
            stash_id: row.try_get("stash_id")?,
            account_name: row.try_get::<Option<String>, &str>("account_name")?,
            league: row.try_get::<Option<String>, &str>("league")?,
            items: row
                .try_get("items")
                .map(serde_json::from_value::<Vec<Item>>)
                .expect("JSON deserialization failed")
                .unwrap(),
        })
    }
}

#[derive(Debug)]
struct StashRecord {
    change_id: String,
    stash_id: String,
    items: Vec<Item>,
    account_name: Option<String>,
    league: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Item {
    id: String,
    type_line: String,
    note: Option<String>,
    stack_size: Option<u32>,
}
