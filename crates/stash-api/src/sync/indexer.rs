use std::time::Duration;
use std::{sync::mpsc::Receiver, sync::mpsc::Sender};

use crate::sync::scheduler::SchedulerMessage;
use crate::{
    common::{poe_ninja_client::PoeNinjaClient, ChangeId, StashTabResponse},
    sync::fetcher::FetchTask,
};

use super::scheduler::start_scheduler;

#[derive(Default)]
pub struct Indexer {
    pub(crate) scheduler_tx: Option<Sender<SchedulerMessage>>,
    pub(crate) is_stopping: bool,
}

impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop(&mut self) {
        self.is_stopping = true;
        log::info!("Stopping indexer");
        self.scheduler_tx
            .as_ref()
            .expect("indexer: Missing ref to scheduler_rx")
            .send(SchedulerMessage::Stop)
            .expect("indexer: Failed to send SchedulerMessage::Stop");
    }

    pub fn is_stopping(&self) -> bool {
        self.is_stopping
    }

    /// Start the indexer with a given change_id
    pub fn start_with_id(&mut self, change_id: ChangeId) -> IndexerResult {
        log::info!("Resuming at change id: {}", change_id);
        self.start(change_id)
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub fn start_with_latest(&mut self) -> IndexerResult {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id()
            .expect("Fetching lastest change_id from poe.ninja failed");
        log::info!("Fetched latest change id: {}", latest_change_id);
        self.start(latest_change_id)
    }

    /// Starts the indexer instance.
    /// This means starting two endlessly running threads:
    ///
    /// a) one thread to fetch the response for a change_id, preemtively deserializing
    ///    the first chunks of it to access the next change_id. It then writes
    ///    the next change_id and the reader of the current response body into
    ///    respective work queues.
    ///
    /// b) another thread with a work queue to deserialize the full response data
    ///    as StashTabResponse structs and sending it to the user of the indexer
    ///    instance.
    fn start(&mut self, change_id: ChangeId) -> IndexerResult {
        let (indexer_rx, scheduler_tx) = start_scheduler();

        scheduler_tx
            .send(SchedulerMessage::Fetch(FetchTask::new(change_id)))
            .expect("indexer: Failed to schedule initial FetchTask");

        self.scheduler_tx = Some(scheduler_tx);

        indexer_rx
    }
}

#[derive(Debug, Clone)]
pub enum IndexerMessage {
    Tick {
        payload: StashTabResponse,
        change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    RateLimited(Duration),
    Stop,
}

type IndexerResult = Receiver<IndexerMessage>;
