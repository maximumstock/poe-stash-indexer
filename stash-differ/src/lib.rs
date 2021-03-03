use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::HashMap;

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff(before: &StashRecord, after: &StashRecord) -> Vec<DiffEvent> {
        let mut events = vec![];

        // Check for removed items
        for before_item in before.items.iter() {
            if after
                .items
                .iter()
                .find(|i| before_item.id.eq(&i.id))
                .is_none()
            {
                events.push(DiffEvent::ItemRemoved(Diff {
                    before: (),
                    after: (),
                    id: before_item.id.clone(),
                    name: before_item.type_line.clone(),
                }));
                continue;
            }
        }

        // Check for added items
        for after_item in after.items.iter() {
            if before
                .items
                .iter()
                .find(|i| after_item.id.eq(&i.id))
                .is_none()
            {
                events.push(DiffEvent::ItemAdded(Diff {
                    before: (),
                    after: (),
                    id: after_item.id.clone(),
                    name: after_item.type_line.clone(),
                }));
                continue;
            }
        }

        // Check for changed items
        for before_item in after.items.iter() {
            if let Some(after_item) = before.items.iter().find(|i| before_item.id.eq(&i.id)) {
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
        }

        events
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

pub struct SampleImporter;

impl SampleImporter {
    pub fn from_file(file_path: &str) -> Vec<StashRecord> {
        let mut reader = ReaderBuilder::new()
            .from_path(file_path)
            .expect("File not found");

        reader
            .deserialize::<HashMap<String, String>>()
            .into_iter()
            .map(|record| {
                let record = record.unwrap();
                StashRecord {
                    stash_id: record.get("stash_id").unwrap().clone(),
                    stash_type: record.get("stash_type").unwrap().clone(),
                    account_name: record.get("account_name").unwrap().clone(),
                    stash_name: record.get("stash_name").unwrap().clone(),
                    league: record.get("league").unwrap().clone(),
                    items: serde_json::from_str::<Vec<Item>>(&record.get("items").unwrap())
                        .unwrap(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct StashRecord {
    stash_id: String,
    stash_type: String,
    items: Vec<Item>,
    account_name: String,
    stash_name: String,
    league: String,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    id: String,
    type_line: String,
    note: Option<String>,
    stack_size: Option<u32>,
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
