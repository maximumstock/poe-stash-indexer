use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::HashMap;

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff(before: &StashRecord, after: &StashRecord) -> Vec<DiffEvent> {
        let mut events = vec![];

        for item in before.items.iter() {
            if after.items.iter().find(|i| item.id.eq(&i.id)).is_none() {
                events.push(DiffEvent::ItemRemoved);
                continue;
            }
        }

        for item in after.items.iter() {
            if before.items.iter().find(|i| item.id.eq(&i.id)).is_none() {
                events.push(DiffEvent::ItemAdded);
                continue;
            }
        }

        events
    }
}

#[derive(Debug)]
pub enum DiffEvent {
    ItemAdded,
    ItemRemoved,
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
}
