use std::collections::HashMap;

use crate::{
    differ::{DiffEvent, StashDiffer},
    stash::{AccountName, AccountStash},
};

pub struct LeagueStore {
    pub inner: HashMap<AccountName, AccountStash>,
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

    pub fn diff_account(&self, account_name: &str, stash: &AccountStash) -> Option<Vec<DiffEvent>> {
        if let Some(previous) = self.inner.get(account_name) {
            let events = StashDiffer::diff_accounts(previous, stash);

            if events.is_empty() {
                None
            } else {
                Some(events)
            }
        } else {
            None
        }
    }

    pub fn update_account(
        &mut self,
        account_name: &str,
        stash: AccountStash,
    ) -> Option<AccountStash> {
        self.inner.insert(account_name.into(), stash)
    }
}
