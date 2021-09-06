use std::ops::AddAssign;

use crate::stash::{AccountStash, Stash};

pub struct StashDiffer;

impl StashDiffer {
    pub fn diff_accounts(before: &AccountStash, after: &AccountStash) -> Vec<DiffEvent> {
        let mut events = vec![];

        for (stash_id, before_stash) in &before.stashes {
            if let Some(after_stash) = after.stashes.get(stash_id) {
                Self::diff_stash(before_stash, after_stash, &mut events);
            }
        }

        events
    }

    pub fn diff_stash(before: &Stash, after: &Stash, buffer: &mut Vec<DiffEvent>) {
        for (item_id, before_item) in before.content.iter() {
            if let Some(after_item) = after.content.get(item_id) {
                // Check for changed notes
                if before_item.note.ne(&after_item.note) {
                    buffer.push(DiffEvent::NoteChanged(Diff {
                        id: after_item.id.clone(),
                        name: after_item.type_line.clone(),
                        before: before_item.note.clone(),
                        after: after_item.note.clone(),
                    }));
                }

                // Check for changed stack_sizes
                if before_item.stack_size.ne(&after_item.stack_size) {
                    buffer.push(DiffEvent::StackSizeChanged(Diff {
                        id: after_item.id.clone(),
                        name: after_item.type_line.clone(),
                        before: before_item.stack_size,
                        after: after_item.stack_size,
                    }));
                }
            } else {
                buffer.push(DiffEvent::Removed(Diff {
                    before: (),
                    after: (),
                    id: before_item.id.clone(),
                    name: before_item.type_line.clone(),
                }));
            }
        }

        for (item_id, after_item) in after.content.iter() {
            if before.content.get(item_id).is_none() {
                buffer.push(DiffEvent::Added(Diff {
                    before: (),
                    after: (),
                    id: after_item.id.clone(),
                    name: after_item.type_line.clone(),
                }));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DiffEvent {
    Added(Diff<()>),
    Removed(Diff<()>),
    NoteChanged(Diff<Option<String>>),
    StackSizeChanged(Diff<Option<u32>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Diff<T> {
    id: String,
    name: String,
    before: T,
    after: T,
}

#[derive(Debug, Copy, Clone, Default)]
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
                DiffEvent::Added(_) => stats.added += 1,
                DiffEvent::Removed(_) => stats.removed += 1,
                DiffEvent::NoteChanged(_) => stats.note += 1,
                DiffEvent::StackSizeChanged(_) => stats.stack_size += 1,
            }
        }

        stats
    }
}
