use std::{borrow::Cow, collections::HashMap};

use chrono::{NaiveDateTime, Utc};
use stash_api::common::{Item, Stash, StashTabResponse};
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

    pub fn ingest(&mut self, incoming: StashTabResponse) -> Vec<DiffEvent> {
        let mut events = vec![];
        let now = Utc::now().naive_local();

        info!("Store: {} stashes", self.inner.len());
        for s in incoming.stashes {
            if s.public {
                if let Some(s) = SearchableStash::from(s, incoming.next_change_id.clone(), now) {
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
    pub account_name: Cow<'static, str>,
    pub id: String,
    pub stash: String,
    pub stash_type: Cow<'static, str>,
    pub items: HashMap<String, Item>,
    pub league: Cow<'static, str>,
    pub timestamp: NaiveDateTime,
    pub change_id: Cow<'static, str>,
}

impl SearchableStash {
    fn from(value: Stash, change_id: String, timestamp: NaiveDateTime) -> Option<Self> {
        if !value.public {
            return None;
        }

        Some(Self {
            account_name: Cow::Owned(value.account_name.unwrap()),
            id: value.id,
            stash: value.stash.unwrap(),
            stash_type: Cow::from(value.stash_type),
            items: value
                .items
                .into_iter()
                .map(|item| (item.id.clone(), item))
                .collect(),
            league: Cow::Owned(value.league.unwrap()),
            timestamp,
            change_id: Cow::from(change_id),
        })
    }
}
