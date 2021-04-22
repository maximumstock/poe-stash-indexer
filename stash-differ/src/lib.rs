use serde::Deserialize;
use sqlx::{postgres::PgRow, Row};
use std::collections::{HashMap, HashSet};

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff(before: &Stash, after: &Stash) -> Vec<DiffEvent> {
        let mut events = vec![];

        let before_item_ids = before.content.keys().collect::<HashSet<_>>();
        let after_item_ids = after.content.keys().collect::<HashSet<_>>();

        let removed_item_ids = before_item_ids.difference(&after_item_ids);
        let added_item_ids = after_item_ids.difference(&before_item_ids);
        let changed_item_ids = after_item_ids.intersection(&before_item_ids);

        // Check for removed items
        for &item_id in removed_item_ids {
            let item = before.content.get(item_id).unwrap();
            events.push(DiffEvent::ItemRemoved(Diff {
                before: (),
                after: (),
                id: item.id.clone(),
                name: item.type_line.clone(),
            }));
        }

        // Check for added items
        for &item_id in added_item_ids {
            let item = after.content.get(item_id).unwrap();
            events.push(DiffEvent::ItemAdded(Diff {
                before: (),
                after: (),
                id: item.id.clone(),
                name: item.type_line.clone(),
            }));
        }

        // Check for changed items
        for &item_id in changed_item_ids {
            let before_item = before.content.get(item_id).unwrap();
            let after_item = after.content.get(item_id).unwrap();

            // Check for changed notes
            if before_item.note.ne(&after_item.note) {
                events.push(DiffEvent::ItemNoteChanged(Diff {
                    id: after_item.id.clone(),
                    name: after_item.type_line.clone(),
                    before: before_item.note.clone(),
                    after: after_item.note.clone(),
                }));
            }

            // Check for changed stack_sizes
            if before_item.stack_size.ne(&after_item.stack_size) {
                events.push(DiffEvent::ItemStackSizeChanged(Diff {
                    id: after_item.id.clone(),
                    name: after_item.type_line.clone(),
                    before: before_item.stack_size,
                    after: after_item.stack_size,
                }));
            }
        }

        events
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
                .try_get("items")
                .map(serde_json::from_value::<Vec<Item>>)
                .expect("JSON deserialization failed")
                .unwrap(),
        })
    }
}

type AccountName = String;
type ItemId = String;
/// The cumulative state of all stashes for a given `account_name`
pub struct Stash {
    account_name: AccountName,
    content: HashMap<ItemId, Item>,
}

impl From<&[StashRecord]> for Stash {
    fn from(stash_records: &[StashRecord]) -> Self {
        Self {
            account_name: stash_records
                .first()
                .unwrap()
                .account_name
                .as_ref()
                .unwrap()
                .clone(),
            content: stash_records
                .iter()
                .flat_map(|sr| &sr.items)
                .cloned()
                .map(|i| (i.id.clone(), i))
                .collect(),
        }
    }
}

pub struct LeagueStore {
    pub inner: HashMap<AccountName, Stash>,
}

impl Default for LeagueStore {
    fn default() -> Self {
        Self {
            inner: HashMap::default(),
        }
    }
}

impl LeagueStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest_account(
        &mut self,
        account_name: &str,
        stash_records: &[StashRecord],
    ) -> Option<Vec<DiffEvent>> {
        let current: Stash = stash_records.into();
        if let Some(previous) = self.inner.get(account_name) {
            // we can diff
            Some(StashDiffer::diff(&previous, &current))
        } else {
            self.inner.insert(account_name.into(), current);
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DiffEvent {
    ItemAdded(Diff<()>),
    ItemRemoved(Diff<()>),
    ItemNoteChanged(Diff<Option<String>>),
    ItemStackSizeChanged(Diff<Option<u32>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Diff<T> {
    id: String,
    name: String,
    before: T,
    after: T,
}

#[derive(Debug)]
pub struct DiffStats {
    pub added: u32,
    pub removed: u32,
    pub note: u32,
    pub stack_size: u32,
}

impl Default for DiffStats {
    fn default() -> Self {
        DiffStats {
            added: 0,
            removed: 0,
            note: 0,
            stack_size: 0,
        }
    }
}
