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

use crate::{change_id::ChangeID, client::RiverClient, types::StashTabResponse};

type BodyQueue = Arc<Mutex<VecDeque<([u8; 70], Box<dyn Read + Send>)>>>;
type ChangeIDQueue = Arc<Mutex<VecDeque<ChangeID>>>;

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

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }
}

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

type IndexerResult = Result<Receiver<usize>, Box<dyn std::error::Error>>;

impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_with_id(&self, change_id: ChangeID) -> IndexerResult {
        self.shared_state
            .change_id_queue
            .lock()
            .unwrap()
            .push_back(change_id);

        self.start()
    }

    pub fn start_with_latest(&self) -> IndexerResult {
        let latest_change_id = RiverClient::fetch_latest_change_id()?;
        println!("Fetched latest change id: {}", latest_change_id);

        self.shared_state
            .change_id_queue
            .lock()
            .unwrap()
            .push_back(latest_change_id);

        self.start()
    }

    fn start(&self) -> IndexerResult {
        let pending_change_ids = self.shared_state.change_id_queue.clone();
        let pending_bodies = self.shared_state.body_queue.clone();
        let pending_bodies2 = self.shared_state.body_queue.clone();
        let (tx, rx) = channel::<usize>();

        let fetcher_handle = std::thread::spawn(move || {
            let mut ratelimit = ratelimit::Builder::new()
                .capacity(2)
                .quantum(2)
                .interval(std::time::Duration::from_millis(1_000))
                .build();

            loop {
                ratelimit.wait();

                let next_change_id = pending_change_ids.lock().unwrap().pop_front().unwrap();

                let start = std::time::Instant::now();
                let url = format!(
                    "http://www.pathofexile.com/api/public-stash-tabs?id={}",
                    next_change_id
                );
                let mut request = ureq::request("GET", &url);
                request.set("Accept-Encoding", "gzip");
                request.set("Accept", "application/json");
                let response = request.call();
                let reader = response.into_reader();

                let mut decoder = GzDecoder::new(BufReader::new(reader));
                let mut next_id_buffer = [0; 70];
                decoder.read_exact(&mut next_id_buffer).unwrap();
                let next_id_raw = next_id_buffer
                    .iter()
                    .skip(19)
                    .take(49)
                    .cloned()
                    .collect::<Vec<u8>>();
                let next_id = String::from_utf8(next_id_raw.clone()).unwrap();
                println!(
                    "Time elapsed - reading next_id: {}ms - next: {}",
                    start.elapsed().as_millis(),
                    next_id
                );

                pending_bodies
                    .lock()
                    .unwrap()
                    .push_back((next_id_buffer, Box::new(decoder)));

                pending_change_ids
                    .lock()
                    .unwrap()
                    .push_back(ChangeID::from_str(&next_id).unwrap());
            }
        });

        let pending_bodies = pending_bodies2;

        let worker_handle = std::thread::spawn(move || loop {
            let mut lock = pending_bodies.lock().unwrap();

            if let Some(next) = lock.pop_front() {
                let (next_id_buffer, mut reader) = next;

                let mut buffer = Vec::new();
                buffer.write_all(&next_id_buffer).unwrap();
                reader.read_to_end(&mut buffer).unwrap();

                let deserialized = serde_json::from_slice::<StashTabResponse>(&buffer).unwrap();

                tx.send(deserialized.stashes.len()).unwrap();
            } else {
                drop(lock);
                std::thread::sleep(std::time::Duration::from_millis(1_000));
            }
        });

        // fetcher_handle.join().expect("Fetcher thread paniced! :O");
        // worker_handle.join().expect("Worker thread paniced! :O");
        Ok(rx)
    }
}
