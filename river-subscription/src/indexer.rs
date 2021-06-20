use flate2::bufread::GzDecoder;
use std::{
    collections::VecDeque,
    io::Write,
    io::{BufReader, Read},
    str::FromStr,
    string::FromUtf8Error,
    sync::mpsc::{channel, Receiver, Sender},
    sync::Arc,
    sync::Mutex,
};

use crate::{change_id::ChangeId, poe_ninja_client::PoeNinjaClient, types::StashTabResponse};

pub struct Indexer {
    shared_state: SharedState,
}

impl Indexer {
    pub fn stop(&mut self) {
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

type SharedState = Arc<Mutex<State>>;

struct State {
    body_queue: BodyQueue,
    change_id_queue: ChangeIdQueue,
    should_stop: bool,
}

impl State {
    fn stop(&mut self) {
        self.should_stop = true;
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            body_queue: VecDeque::new(),
            change_id_queue: VecDeque::new(),
            should_stop: false,
        }
    }
}

type ChangeIdRequest = (ChangeId, usize);
type ChangeIdQueue = VecDeque<ChangeIdRequest>;
type BodyQueue = VecDeque<WorkerTask>;

struct WorkerTask {
    fetch_partial: [u8; 80],
    change_id: ChangeId,
    reader: Box<dyn Read + Send>,
}

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
            .change_id_queue
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

        rx
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
            if shared_state.lock().unwrap().should_stop {
                break;
            }

            ratelimit.wait();

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
            let mut next_id_buffer = [0; 80];

            match decoder.read_exact(&mut next_id_buffer) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    log::error!("UnexpectedEof: {:?}", next_id_buffer);
                    shared_state.lock().unwrap().stop();
                    continue;
                }
                Err(err) => {
                    log::error!("fetcher: gzip decoding failed: {}", err);

                    match reschedule(shared_state.clone(), change_id_request.clone()) {
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
            }

            let next_id = parse_change_id_from_bytes(&next_id_buffer)
                .expect("Preemptive deserialization of next change_id failed");

            log::debug!(
                "Took {}ms to read next id: {}",
                start.elapsed().as_millis(),
                next_id
            );

            let next_change_id = ChangeId::from_str(&next_id).expect("Invalid change_id provided");

            let next_worker_task = WorkerTask {
                reader: Box::new(decoder),
                fetch_partial: next_id_buffer,
                change_id,
            };

            let mut lock = shared_state.lock().unwrap();
            lock.body_queue.push_back(next_worker_task);
            lock.change_id_queue.push_back((next_change_id, 0));
            drop(lock);
        }

        shared_state.lock().unwrap().stop();
    })
}

fn reschedule(shared_state: SharedState, request: ChangeIdRequest) -> Result<(), ()> {
    if request.1 > 2 {
        log::error!("Retried too many times...shutting down");
        return Err(());
    }

    let new_request = (request.0, request.1 + 1);
    log::info!(
        "Rescheduling {} (Retried {} times)",
        new_request.0,
        request.1
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

        if lock.should_stop {
            tx.send(IndexerMessage::Stop).unwrap();
            break;
        }

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

            let msg = IndexerMessage::Tick {
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

fn parse_change_id_from_bytes(bytes: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(
        bytes
            .split(|b| (*b as char).eq(&'"'))
            .nth(3)
            .unwrap()
            .to_vec(),
    )
}

#[cfg(test)]
mod test {
    use super::parse_change_id_from_bytes;

    #[test]
    fn test_parse_change_id_from_bytes() {
        let input = "{\"next_change_id\": \"abc-def-ghi-jkl-mno\", \"stashes\": []}".as_bytes();
        let result = parse_change_id_from_bytes(&input);
        assert_eq!(result, Ok("abc-def-ghi-jkl-mno".into()));
    }
}
