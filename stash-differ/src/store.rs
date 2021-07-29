use std::collections::HashMap;

use crate::{
    differ::{DiffEvent, StashDiffer},
    stash::{AccountName, Stash},
};

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
        if let Some(previous) = self.inner.get(account_name) {
            let events = StashDiffer::diff(previous, stash);

            if events.is_empty() {
                None
            } else {
                Some(events)
            }
        } else {
            None
        }
    }

    pub fn update_account(&mut self, account_name: &str, stash: Stash) -> Option<Stash> {
        self.inner.insert(account_name.into(), stash)
    }
}
