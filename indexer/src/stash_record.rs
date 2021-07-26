use std::time::SystemTime;

use crate::schema::stash_records;
use chrono::{DateTime, NaiveDateTime, Utc};
use river_subscription::{common::ChangeId, common::StashTabResponse};
use serde::Serialize;

#[derive(Serialize, Insertable, Queryable)]
#[table_name = "stash_records"]
pub struct StashRecord {
    pub created_at: NaiveDateTime,
    pub change_id: String,
    pub next_change_id: String,
    pub stash_id: String,
    pub stash_type: String,
    pub items: serde_json::Value,
    pub public: bool,
    pub account_name: Option<String>,
    pub last_character_name: Option<String>,
    pub stash_name: Option<String>,
    pub league: Option<String>,
    pub chunk_id: i64,
}

pub fn map_to_stash_records(
    change_id: ChangeId,
    created_at: SystemTime,
    payload: StashTabResponse,
    chunk_id: i64,
) -> Vec<StashRecord> {
    let next_change_id = payload.next_change_id;

    payload
        .stashes
        .into_iter()
        // Ignore stash tabs flagged as private, whose updates are always empty
        .filter(|stash| stash.public)
        .map(move |stash| StashRecord {
            account_name: stash.account_name,
            last_character_name: stash.last_character_name,
            stash_id: stash.id,
            stash_name: stash.stash,
            stash_type: stash.stash_type,
            items: serde_json::to_value(stash.items).expect("Serialization failed"),
            public: stash.public,
            league: stash.league,
            change_id: change_id.clone().into(),
            created_at: DateTime::<Utc>::from(created_at).naive_utc(),
            next_change_id: next_change_id.clone(),
            chunk_id,
        })
        .collect::<Vec<_>>()
}
