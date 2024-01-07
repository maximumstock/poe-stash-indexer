use serde::Deserialize;
use sqlx::{postgres::PgRow, Row};
use std::collections::HashMap;

pub type AccountName = String;
pub type ItemId = String;
pub type StashId = String;

/// The cumulative state of all stashes for a given `account_name`
#[derive(Debug, Default)]
pub struct AccountStash {
    pub account_name: Option<AccountName>,
    pub league: Option<String>,
    pub stashes: HashMap<StashId, Stash>,
    pub update_count: u64,
}

impl AccountStash {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_stash(mut self, stash_id: impl Into<StashId>, stash: Stash) -> Self {
        self.stashes.insert(stash_id.into(), stash);
        self
    }
}

#[derive(Debug, Default)]
pub struct Stash {
    pub stash_id: StashId,
    pub content: HashMap<ItemId, Item>,
    pub update_count: u64,
}

impl Stash {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_item(mut self, item_id: impl Into<ItemId>, item: Item) -> Self {
        self.content.insert(item_id.into(), item);
        self
    }
}

impl From<Vec<StashRecord>> for AccountStash {
    fn from(stash_records: Vec<StashRecord>) -> Self {
        let account_name = stash_records.first().unwrap().account_name.clone();
        let league = stash_records.first().unwrap().league.clone();

        Self {
            account_name,
            league,
            stashes: stash_records
                .into_iter()
                .map(|sr| (sr.stash_id.clone(), sr.into()))
                .collect(),
            update_count: 0,
        }
    }
}

impl From<StashRecord> for Stash {
    fn from(stash_record: StashRecord) -> Self {
        Self {
            stash_id: stash_record.stash_id,
            content: stash_record
                .items
                .into_iter()
                .map(|i| (i.id.clone(), i))
                .collect(),
            update_count: 0,
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
    pub public: bool,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Item {
    pub id: String,
    pub type_line: String,
    pub note: Option<String>,
    pub stack_size: Option<u32>,
    pub birthday: Option<u64>,
}

impl Item {
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, type_line: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            type_line: type_line.into(),
            ..Self::default()
        }
    }
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
            public: row.try_get("public")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

pub fn group_stash_records_by_account_name(
    stash_records: Vec<StashRecord>,
) -> HashMap<String, Vec<StashRecord>> {
    let mut stash_records_by_account_name = HashMap::new();

    for sr in stash_records {
        if let Some(account_name) = &sr.account_name {
            stash_records_by_account_name
                .entry(account_name.clone())
                .or_insert_with(Vec::new)
                .push(sr)
        }
    }

    stash_records_by_account_name
}

#[cfg(test)]
mod tests {

    use crate::{differ::StashDiffer, stash::Stash};

    use super::{AccountStash, Item};

    #[test]
    fn test_diffing_happens_on_stash_level() {
        let account_before = AccountStash::new().with_stash(
            "Stash1",
            Stash::new().with_item("Item A", Item::new("Item A", "Unique")),
        );
        let account_after = AccountStash::new().with_stash("Stash1", Stash::new());

        let events = StashDiffer::diff_accounts(&account_before, &account_after);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_diffing_does_not_happen_on_account_level() {
        let account_before = AccountStash::new().with_stash(
            "Stash1",
            Stash::new().with_item("Item A", Item::new("Item A", "Unique")),
        );
        let account_after = AccountStash::new().with_stash("Stash2", Stash::new());

        let events = StashDiffer::diff_accounts(&account_before, &account_after);
        assert!(events.is_empty());
    }
}
