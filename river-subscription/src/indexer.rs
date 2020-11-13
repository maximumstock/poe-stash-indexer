use flate2::bufread::GzDecoder;
use std::{
    collections::VecDeque,
    io::Write,
    io::{BufReader, Read},
    str::FromStr,
    sync::mpsc::{channel, Receiver, Sender},
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
            shared_state: Arc::new(Mutex::new(State::default())),
        }
    }
}

type SharedState = Arc<Mutex<State>>;

struct State {
    body_queue: BodyQueue,
    change_id_queue: ChangeIDQueue,
}

impl Default for State {
    fn default() -> Self {
        Self {
            body_queue: VecDeque::new(),
            change_id_queue: VecDeque::new(),
        }
    }
}

type ChangeIDRequest = (ChangeID, usize);
type ChangeIDQueue = VecDeque<ChangeIDRequest>;
type BodyQueue = VecDeque<WorkerTask>;

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
            .lock()
            .unwrap()
            .change_id_queue
            .push_back((change_id, 0));

        self.start()
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub fn start_with_latest(&self) -> IndexerResult {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id()?;
        log::info!("Fetched latest change id: {}", latest_change_id);

        self.shared_state
            .lock()
            .unwrap()
            .change_id_queue
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

        Ok(rx)
    }
}

fn start_fetcher(shared_state: SharedState) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // Break down rate-limit into quantum of 1, so we never do any bursts,
        // like we would with for example 2 requests per second.
        let mut ratelimit = ratelimit::Builder::new()
            .capacity(1)
            .quantum(1)
            .interval(std::time::Duration::from_millis(500))
            .build();

        loop {
            let change_id_request = shared_state
                .lock()
                .unwrap()
                .change_id_queue
                .pop_front()
                .unwrap();

            let (change_id, _) = change_id_request.clone();

            let start = std::time::Instant::now();
            let url = format!(
                "http://www.pathofexile.com/api/public-stash-tabs?id={}",
                change_id
            );

            log::debug!("Requesting {}", change_id);
            let mut request = ureq::request("GET", &url);
            request.set("Accept-Encoding", "gzip");
            request.set("Accept", "application/json");
            let response = request.call();

            if response.error() {
                log::error!("fetcher: HTTP error {}", response.status());
                log::error!("fetcher: HTTP response: {:?}", response);

                match reschedule(shared_state.clone(), change_id_request) {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }

            let reader = response.into_reader();

            let mut decoder = GzDecoder::new(BufReader::new(reader));
            let mut next_id_buffer = [0; 70];
            let decoded = decoder.read_exact(&mut next_id_buffer);

            if decoded.is_err() {
                log::error!("fetcher: gzip decoding failed: {}", decoded.unwrap_err());

                match reschedule(shared_state.clone(), change_id_request) {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }

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

            let next_change_id = ChangeID::from_str(&next_id).expect("Invalid change_id provided");

            let next_worker_task = WorkerTask {
                reader: Box::new(decoder),
                fetch_partial: next_id_buffer,
                change_id,
            };

            let mut lock = shared_state.lock().unwrap();
            lock.body_queue.push_back(next_worker_task);
            lock.change_id_queue.push_back((next_change_id, 0));
            drop(lock);

            ratelimit.wait();
        }
    })
}

fn reschedule(shared_state: SharedState, request: ChangeIDRequest) -> Result<(), ()> {
    if request.1 > 2 {
        log::error!("Retried too many times...shutting down");
        return Err(());
    }

    let new_request = (request.0, request.1 + 1);
    log::info!(
        "Rescheduling {} (Retried {} times)",
        new_request.0,
        new_request.1
    );
    shared_state
        .lock()
        .unwrap()
        .change_id_queue
        .push_back(new_request);

    Ok(())
}

fn start_worker(
    shared_state: SharedState,
    tx: Sender<IndexerMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let mut lock = shared_state.lock().unwrap();

        if let Some(next) = lock.body_queue.pop_front() {
            let mut task = next;

            let start = std::time::Instant::now();
            let mut buffer = Vec::new();
            buffer.write_all(&task.fetch_partial).unwrap();
            task.reader.read_to_end(&mut buffer).unwrap();

            let deserialized = serde_json::from_slice::<StashTabResponse>(&buffer)
                .expect("Deserialization of body failed");
            log::debug!(
                "Took {}ms to read & deserialize body",
                start.elapsed().as_millis()
            );

            let msg = IndexerMessage {
                payload: deserialized,
                change_id: task.change_id,
                created_at: std::time::SystemTime::now(),
            };

            tx.send(msg).expect("Sending IndexerMessage failed");
        } else {
            drop(lock);
            log::debug!("Worker is waiting due to no work");
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    })
}
