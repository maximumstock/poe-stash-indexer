use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::stash::StashRecord;

mod aggregation;
mod flat;

pub use aggregation::aggregation_consumer;
pub use flat::flat_consumer;

#[derive(Default)]
pub struct State {
    pub queue: VecDeque<Vec<StashRecord>>,
}

pub type SharedState = Arc<(Mutex<State>, Condvar)>;
