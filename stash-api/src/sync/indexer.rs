use std::{
    sync::mpsc::Sender,
    sync::mpsc::{channel, Receiver},
};

use crate::sync::{poe_ninja_client::PoeNinjaClient, scheduler::SchedulerMessage};
use crate::{
    common::{ChangeId, StashTabResponse},
    sync::fetcher::FetchTask,
};

use super::{
    fetcher::{start_fetcher, FetcherMessage},
    scheduler::start_scheduler,
    worker::{start_worker, WorkerMessage},
};

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
        let (scheduler_tx, scheduler_rx) = channel::<SchedulerMessage>();
        let (fetcher_tx, fetcher_rx) = channel::<FetcherMessage>();
        let (worker_tx, worker_rx) = channel::<WorkerMessage>();
        let (indexer_tx, indexer_rx) = channel::<IndexerMessage>();

        let _fetcher_handle = start_fetcher(fetcher_rx, scheduler_tx.clone(), worker_tx);
        let _worker_handle = start_worker(worker_rx, scheduler_tx.clone(), indexer_tx);
        let _scheduler_handle = start_scheduler(scheduler_rx, fetcher_tx);

        scheduler_tx
            .send(SchedulerMessage::Task(FetchTask::new(change_id)))
            .expect("indexer: Failed to schedule initial FetchTask");

        self.scheduler_tx = Some(scheduler_tx);

        indexer_rx
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum IndexerMessage {
    Tick {
        payload: StashTabResponse,
        change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    Stop,
}

type IndexerResult = Receiver<IndexerMessage>;

#[cfg(test)]
mod test {
    use std::{sync::mpsc::RecvTimeoutError, time::Duration};

    use crate::sync::fetcher::parse_change_id_from_bytes;

    use super::{Indexer, IndexerMessage};

    #[test]
    fn test_parse_change_id_from_bytes() {
        let input = "{\"next_change_id\": \"abc-def-ghi-jkl-mno\", \"stashes\": []}".as_bytes();
        let result = parse_change_id_from_bytes(input);
        assert_eq!(result, Ok("abc-def-ghi-jkl-mno".into()));
    }

    #[test]
    fn test_indexer() {
        let mut indexer = Indexer::new();
        let rx = indexer.start_with_latest();
        std::thread::sleep(Duration::from_secs(3));
        indexer.stop();

        let (mut n_tick, mut n_stop) = (0, 0);

        while let Ok(msg) = rx.recv() {
            match msg {
                IndexerMessage::Stop => n_stop += 1,
                IndexerMessage::Tick { .. } => n_tick += 1,
            }
        }

        assert!(n_tick > 0);
        assert_eq!(n_stop, 1);
        assert_eq!(
            Err(RecvTimeoutError::Disconnected),
            rx.recv_timeout(Duration::from_secs(10))
        );
    }
}