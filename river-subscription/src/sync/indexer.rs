use std::{
    io::Read,
    sync::mpsc::{channel, Receiver},
    sync::Arc,
    sync::Mutex,
};

use crate::common::{ChangeId, StashTabResponse};
use crate::sync::poe_ninja_client::PoeNinjaClient;

use super::{
    fetcher::start_fetcher,
    state::{SharedState, State},
    worker::start_worker,
};

pub struct Indexer {
    pub(crate) shared_state: SharedState,
}

impl Indexer {
    pub fn stop(&mut self) {
        println!("Stopping indexer");
        self.shared_state.lock().unwrap().should_stop = true;
    }

    pub fn is_stopping(&self) -> bool {
        self.shared_state.lock().unwrap().should_stop
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(State::default())),
        }
    }
}

pub(crate) struct WorkerTask {
    pub(crate) fetch_partial: [u8; 80],
    pub(crate) change_id: ChangeId,
    pub(crate) reader: Box<dyn Read + Send>,
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

impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the indexer with a given change_id
    pub fn start_with_id(&self, change_id: ChangeId) -> IndexerResult {
        log::info!("Resuming at change id: {}", change_id);

        self.shared_state
            .lock()
            .unwrap()
            .fetcher_queue
            .push_back((change_id, 0));

        self.start()
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub fn start_with_latest(&self) -> IndexerResult {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id()
            .expect("Fetching lastest change_id from poe.ninja failed");
        log::info!("Fetched latest change id: {}", latest_change_id);

        self.shared_state
            .lock()
            .unwrap()
            .fetcher_queue
            .push_back((latest_change_id, 0));

        self.start()
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
    fn start(&self) -> IndexerResult {
        let (tx, rx) = channel::<IndexerMessage>();

        let _fetcher_handle = start_fetcher(self.shared_state.clone());
        let _worker_handle = start_worker(self.shared_state.clone(), tx);

        rx
    }
}

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
