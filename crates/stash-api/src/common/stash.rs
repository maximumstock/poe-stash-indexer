use chrono::NaiveDateTime;
use serde::Serialize;

use crate::poe_api::poe_stash_api::protocol::Item;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Stash {
    pub id: String,
    pub public: bool,
    pub account_name: Option<String>,
    pub stash: Option<String>,
    pub stash_type: String,
    pub items: Vec<Item>,
    pub league: Option<String>,
    pub created_at: NaiveDateTime,
    pub change_id: String,
    pub next_change_id: String,
}
