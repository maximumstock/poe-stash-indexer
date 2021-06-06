use std::ops::AddAssign;

use crate::stash::Stash;

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff(before: &Stash, after: &Stash) -> Vec<DiffEvent> {
        let mut events = vec![];

        for (item_id, before_item) in before.content.iter() {
            if let Some(after_item) = after.content.get(item_id) {
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
            } else {
                events.push(DiffEvent::ItemRemoved(Diff {
                    before: (),
                    after: (),
                    id: before_item.id.clone(),
                    name: before_item.type_line.clone(),
                }));
            }
        }

        for (item_id, after_item) in after.content.iter() {
            if before.content.get(item_id).is_none() {
                events.push(DiffEvent::ItemAdded(Diff {
                    before: (),
                    after: (),
                    id: after_item.id.clone(),
                    name: after_item.type_line.clone(),
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
