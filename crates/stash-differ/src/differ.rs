use std::borrow::Cow;

use chrono::NaiveDateTime;
use serde::Serialize;
use stash_api::common::Item;
use tracing::info;

use crate::store::SearchableStash;

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff_stash(
        before: &SearchableStash,
        after: &SearchableStash,
        buffer: &mut Vec<DiffEvent>,
    ) {
        if !after.public {
            // Avoid non-public stashes, since its optional data fields are null anyway
            return;
        }

        // We know now that all the optional fields are set
        info!("Diffing stash {}", before.id);
        for (item_id, before_item) in before.items.iter() {
            if let Some(after_item) = after.items.get(item_id) {
                // Check for changed stack_sizes
                let stack_size_changed = before_item.stack_size.ne(&after_item.stack_size);
                // Check for changed notes
                let note_changed = before_item.note.ne(&after_item.note);

                if note_changed || stack_size_changed {
                    buffer.push(DiffEvent::Changed(Changed {
                        old: before_item.clone(),
                        new: after_item.clone(),
                        note_changed,
                        stack_size_changed,
                        meta: DiffMeta {
                            league: before.league.clone().expect("league option empty"),
                            account_name: before
                                .account_name
                                .clone()
                                .expect("account name option empty"),
                            stash_type: before.stash_type.clone(),
                            old_change_id: before.change_id.clone(),
                            new_change_id: after.change_id.clone(),
                            old_timestamp: before.timestamp,
                            new_timestamp: after.timestamp,
                        },
                    }));
                }
            } else {
                buffer.push(DiffEvent::Removed(Removed {
                    item: before_item.clone(),
                    meta: DiffMeta {
                        league: before.league.clone().expect("league option empty"),
                        account_name: before
                            .account_name
                            .clone()
                            .expect("account name option empty"),
                        stash_type: before.stash_type.clone(),
                        old_change_id: before.change_id.clone(),
                        new_change_id: after.change_id.clone(),
                        old_timestamp: before.timestamp,
                        new_timestamp: after.timestamp,
                    },
                }));
            }
        }

        for (item_id, after_item) in after.items.iter() {
            if before.items.get(item_id).is_none() {
                buffer.push(DiffEvent::Added(Added {
                    item: after_item.clone(),
                    meta: DiffMeta {
                        league: before.league.clone().expect("league option empty"),
                        account_name: before
                            .account_name
                            .clone()
                            .expect("account name option empty"),
                        stash_type: before.stash_type.clone(),
                        old_change_id: before.change_id.clone(),
                        new_change_id: after.change_id.clone(),
                        old_timestamp: before.timestamp,
                        new_timestamp: after.timestamp,
                    },
                }));
            }
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum DiffEvent {
    Added(Added),
    Removed(Removed),
    Changed(Changed),
}

impl DiffEvent {
    pub fn timestamp(&self) -> NaiveDateTime {
        match self {
            DiffEvent::Added(added) => added.meta.new_timestamp,
            DiffEvent::Removed(removed) => removed.meta.new_timestamp,
            DiffEvent::Changed(changed) => changed.meta.new_timestamp,
        }
    }

    pub fn league(&self) -> &str {
        match self {
            DiffEvent::Added(added) => &added.meta.league,
            DiffEvent::Removed(removed) => &removed.meta.league,
            DiffEvent::Changed(changed) => &changed.meta.league,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Added {
    item: Item,
    meta: DiffMeta,
}

#[derive(Serialize, Clone)]
pub struct Removed {
    item: Item,
    meta: DiffMeta,
}

#[derive(Serialize, Clone)]
pub struct Changed {
    old: Item,
    new: Item,
    note_changed: bool,
    stack_size_changed: bool,
    meta: DiffMeta,
}

#[derive(Serialize, Clone)]
pub struct DiffMeta {
    league: Cow<'static, str>,
    account_name: Cow<'static, str>,
    stash_type: Cow<'static, str>,
    old_change_id: Cow<'static, str>,
    new_change_id: Cow<'static, str>,
    old_timestamp: NaiveDateTime,
    new_timestamp: NaiveDateTime,
}
