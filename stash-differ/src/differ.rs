use std::{collections::HashSet, ops::AddAssign};

use crate::stash::Stash;

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

#[derive(Debug, Copy, Clone)]
pub struct DiffStats {
    pub added: u32,
    pub removed: u32,
    pub note: u32,
    pub stack_size: u32,
}

impl AddAssign for DiffStats {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            added: self.added + rhs.added,
            removed: self.removed + rhs.removed,
            note: self.note + rhs.note,
            stack_size: self.stack_size + rhs.stack_size,
        }
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
