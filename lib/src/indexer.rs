use flate2::bufread::GzDecoder;
use std::{
    collections::VecDeque,
    io::Write,
    io::{BufReader, Read},
    str::FromStr,
    sync::mpsc::{channel, Receiver},
    sync::Arc,
    sync::Mutex,
};

use crate::{change_id::ChangeID, poe_ninja_client::PoeNinjaClient, types::StashTabResponse};

pub struct Indexer {
    shared_state: SharedState,
}

impl Default for Indexer {
    fn default() -> Self {
        Self {
            shared_state: SharedState::default(),
        }
    }
}

#[derive(Clone)]
struct SharedState {
    body_queue: BodyQueue,
    change_id_queue: ChangeIDQueue,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            body_queue: Arc::new(Mutex::new(VecDeque::new())),
            change_id_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

type ChangeIDQueue = Arc<Mutex<VecDeque<ChangeID>>>;
type BodyQueue = Arc<Mutex<VecDeque<WorkerTask>>>;

struct WorkerTask {
    fetch_partial: [u8; 70],
    change_id: ChangeID,
    reader: Box<dyn Read + Send>,
}

pub struct IndexerMessage {
    pub payload: StashTabResponse,
    pub change_id: ChangeID,
    pub created_at: std::time::SystemTime,
}

type IndexerResult = Result<Receiver<IndexerMessage>, Box<dyn std::error::Error>>;

impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the indexer with a given change_id
    pub fn start_with_id(&self, change_id: ChangeID) -> IndexerResult {
        self.shared_state
            .change_id_queue
            .lock()
            .unwrap()
            .push_back(change_id);

        self.start()
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub fn start_with_latest(&self) -> IndexerResult {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id()?;
        log::info!("Fetched latest change id: {}", latest_change_id);

        self.shared_state
            .change_id_queue
            .lock()
            .unwrap()
            .push_back(latest_change_id);

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

        let pending_change_ids = self.shared_state.change_id_queue.clone();
        let pending_bodies = self.shared_state.body_queue.clone();

        let _fetcher_handle = std::thread::spawn(move || {
            // Break down rate-limit into quantum of 1, so we never do any bursts,
            // like we would with for example 2 requests per second.
            let mut ratelimit = ratelimit::Builder::new()
                .capacity(1)
                .quantum(1)
                .interval(std::time::Duration::from_millis(500))
                .build();

            loop {
                ratelimit.wait();

                let change_id = pending_change_ids.lock().unwrap().pop_front().unwrap();

                let start = std::time::Instant::now();
                let url = format!(
                    "http://www.pathofexile.com/api/public-stash-tabs?id={}",
                    change_id
                );
                let mut request = ureq::request("GET", &url);
                request.set("Accept-Encoding", "gzip");
                request.set("Accept", "application/json");
                let response = request.call();
                let reader = response.into_reader();

                let mut decoder = GzDecoder::new(BufReader::new(reader));
                let mut next_id_buffer = [0; 70];
                decoder.read_exact(&mut next_id_buffer).unwrap();
                let next_id = String::from_utf8(
                    next_id_buffer
                        .iter()
                        .skip(19)
                        .take(49)
                        .cloned()
                        .collect::<Vec<u8>>(),
                )
                .expect("Preemptive deserialization of next change_id failed");

                log::debug!(
                    "Took {}ms to read next id: {}",
                    start.elapsed().as_millis(),
                    next_id
                );

                let next_change_id =
                    ChangeID::from_str(&next_id).expect("Invalid change_id provided");

                let next_worker_task = WorkerTask {
                    reader: Box::new(decoder),
                    fetch_partial: next_id_buffer,
                    change_id,
                };

                pending_bodies.lock().unwrap().push_back(next_worker_task);

                pending_change_ids.lock().unwrap().push_back(next_change_id);
            }
        });

        let pending_bodies = self.shared_state.body_queue.clone();

        let _worker_handle = std::thread::spawn(move || loop {
            let mut lock = pending_bodies.lock().unwrap();

            if let Some(next) = lock.pop_front() {
                let mut task = next;

                let start = std::time::Instant::now();
                let mut buffer = Vec::new();
                buffer.write_all(&task.fetch_partial).unwrap();
                task.reader.read_to_end(&mut buffer).unwrap();

                let deserialized = serde_json::from_slice::<StashTabResponse>(&buffer).unwrap();
                log::debug!(
                    "Took {}ms to read & deserialize body",
                    start.elapsed().as_millis()
                );

                let msg = IndexerMessage {
                    payload: deserialized,
                    change_id: task.change_id,
                    created_at: std::time::SystemTime::now(),
                };

                tx.send(msg).unwrap();
            } else {
                drop(lock);
                log::debug!("Worker is waiting due to no work");
                std::thread::sleep(std::time::Duration::from_millis(1_000));
            }
        });

        // fetcher_handle.join().expect("Fetcher thread paniced! :O");
        // worker_handle.join().expect("Worker thread paniced! :O");
        Ok(rx)
    }
}
