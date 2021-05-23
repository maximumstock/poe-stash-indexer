use serde::Deserialize;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use std::{collections::{HashMap, HashSet, VecDeque}, iter::Sum};

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

type AccountName = String;
type ItemId = String;
/// The cumulative state of all stashes for a given `account_name`
pub struct Stash {
    content: HashMap<ItemId, Item>,
}

impl From<&[StashRecord]> for Stash {
    fn from(stash_records: &[StashRecord]) -> Self {
        Self {
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

    pub fn diff_account(&self, account_name: &str, stash: &Stash) -> Option<Vec<DiffEvent>> {
        self.inner
            .get(account_name)
            .map(|previous| StashDiffer::diff(&previous, &stash))
    }

    pub fn update_account(&mut self, account_name: &str, stash: Stash) -> Option<Stash> {
        self.inner.insert(account_name.into(), stash)
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

impl<'a> Sum<&'a DiffStats> for DiffStats {
    fn sum<I: Iterator<Item = &'a DiffStats>>(iter: I) -> Self {
        let mut stats = DiffStats::default();

        for ds in iter {
            stats.added += ds.added;
            stats.removed += ds.removed;
            stats.note += ds.note;
            stats.stack_size += ds.stack_size;
        }

        stats
    }
}

impl From<&[DiffEvent]> for DiffStats {
    fn from(events: &[DiffEvent]) -> Self {
        let mut stats = DiffStats::default();

        for ev in events {
            match ev {
                DiffEvent::ItemAdded(_) => stats.added += 1,
                DiffEvent::ItemRemoved(_) => stats.removed += 1,
                DiffEvent::ItemNoteChanged(_) => stats.note += 1,
                DiffEvent::ItemStackSizeChanged(_) => stats.stack_size += 1,
            }
        }

        stats
    }
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

pub struct StashRecordIterator<'a> {
    pool: &'a Pool<Postgres>,
    runtime: &'a tokio::runtime::Runtime,
    league: &'a str,
    page_size: i64,
    page: (i64, i64),
    buffer: VecDeque<StashRecord>,
    available_chunks: usize,
}

impl<'a> StashRecordIterator<'a> {
    pub fn new(
        pool: &'a Pool<Postgres>,
        runtime: &'a tokio::runtime::Runtime,
        page_size: i64,
        league: &'a str,
    ) -> Self {
        Self {
            pool,
            runtime,
            league,
            page_size,
            page: (0, page_size),
            buffer: VecDeque::new(),
            available_chunks: 0,
        }
    }

    fn needs_data(&self) -> bool {
        self.available_chunks < 2
    }

    fn count_available_chunks(&self) -> usize {
        self.buffer
            .iter()
            .map(|i| &i.change_id)
            .collect::<HashSet<_>>()
            .len()
    }

    fn load_data(&mut self) -> Result<(), sqlx::Error> {
        let next_page = self.runtime.block_on(fetch_stash_records_paginated(
            &self.pool,
            self.page.0,
            self.page.1,
            self.league,
        ))?;

        self.buffer.extend(next_page);
        self.page = (self.page.1, self.page.1 + self.page_size);
        self.available_chunks = self.count_available_chunks();
        Ok(())
    }

    fn extract_first_chunk(&mut self) -> Vec<StashRecord> {
        if self.needs_data() {
            panic!("Expected to have more data");
        }

        let next_change_id = &self
            .buffer
            .front()
            .expect("No data where some was expected")
            .change_id
            .clone();

        let mut data = vec![];

        while let Some(next) = self.buffer.front() {
            if next.change_id.eq(next_change_id) {
                let v = self.buffer.pop_front().expect("taking first stash record from queue");
                data.push(v);
            } else {
                break;
            }
        }

        self.available_chunks -= 1;

        data
    }

    pub fn next_chunk(&mut self) -> Option<Vec<StashRecord>> {
        while self.needs_data() {
            self.load_data().expect("Fetching next page failed");
        }

        Some(self.extract_first_chunk())
    }
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

pub fn group_stash_records_by_account_name(
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
