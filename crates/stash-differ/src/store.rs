use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use stash_api::{common::stash::Stash, poe_api::poe_stash_api::protocol::Item};
use tracing::info;

use crate::differ::{DiffEvent, StashDiffer};

type StashId = String;

#[derive(Default)]
pub struct StashStore {
    pub inner: HashMap<StashId, SearchableStash>,
}

impl StashStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, incoming: Vec<Stash>, next_change_id: String) -> Vec<DiffEvent> {
        let mut events = vec![];
        let now = Utc::now().naive_local();

        info!("Store: {} stashes", self.inner.len());
        for s in incoming {
            if s.public {
                if let Some(s) = SearchableStash::from(s, next_change_id.clone(), now) {
                    if let Some(previous) = self.inner.get(&s.id) {
                        StashDiffer::diff_stash(previous, &s, &mut events);
                    }
                    self.inner.insert(s.id.clone(), s);
                }
            } else {
                self.inner.remove(&s.id);
            }
        }

        events
    }
}

pub struct SearchableStash {
    pub account_name: Option<String>,
    pub id: String,
    pub stash_type: String,
    pub items: HashMap<String, Item>,
    pub league: Option<String>,
    pub timestamp: NaiveDateTime,
    pub change_id: String,
}

impl SearchableStash {
    fn from(value: Stash, change_id: String, timestamp: NaiveDateTime) -> Option<Self> {
        if !value.public {
            return None;
        }

        Some(Self {
            account_name: value.account_name,
            id: value.id,
            stash_type: value.stash_type,
            items: value
                .items
                .into_iter()
                .map(|item| (item.id.clone().unwrap(), item))
                .collect(),
            league: value.league,
            timestamp,
            change_id,
        })
    }
}
