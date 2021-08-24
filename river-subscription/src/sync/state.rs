use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::common::ChangeId;

use super::indexer::WorkerTask;

pub(crate) type SharedState = Arc<Mutex<State>>;

pub(crate) struct State {
    pub(crate) worker_queue: WorkerQueue,
    pub(crate) fetcher_queue: FetcherQueue,
    pub(crate) should_stop: bool,
}

impl State {
    pub fn stop(&mut self) {
        self.should_stop = true;
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            worker_queue: VecDeque::new(),
            fetcher_queue: VecDeque::new(),
            should_stop: false,
        }
    }
}

pub(crate) type ChangeIdRequest = (ChangeId, usize);
pub(crate) type FetcherQueue = VecDeque<ChangeIdRequest>;
pub(crate) type WorkerQueue = VecDeque<WorkerTask>;
