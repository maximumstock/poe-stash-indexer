use serde::Deserialize;
use sqlx::{postgres::PgRow, Row};
use std::collections::HashMap;

pub type AccountName = String;
pub type ItemId = String;

/// The cumulative state of all stashes for a given `account_name`
pub struct Stash {
    pub content: HashMap<ItemId, Item>,
}

impl From<Vec<StashRecord>> for Stash {
    fn from(stash_records: Vec<StashRecord>) -> Self {
        Self {
            content: stash_records
                .into_iter()
                .flat_map(|sr| sr.items)
                .map(|i| (i.id.clone(), i))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StashRecord {
    pub change_id: String,
    pub next_change_id: String,
    pub stash_id: String,
    pub items: Vec<Item>,
    pub account_name: Option<String>,
    pub league: Option<String>,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: String,
    pub type_line: String,
    pub note: Option<String>,
    pub stack_size: Option<u32>,
}

impl<'r> sqlx::FromRow<'r, PgRow> for StashRecord {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(StashRecord {
            change_id: row.try_get("change_id")?,
            next_change_id: row.try_get("next_change_id")?,
            stash_id: row.try_get("stash_id")?,
            account_name: row.try_get::<Option<String>, &str>("account_name")?,
            league: row.try_get::<Option<String>, &str>("league")?,
            items: row
                .try_get::<sqlx::types::Json<Vec<Item>>, &str>("items")?
                .0,
            created_at: row.try_get("created_at")?,
        })
    }
}
