use flate2::bufread::GzDecoder;

use super::change_id::ChangeID;
use crate::client::RiverClient;
use crate::parser::StashTabResponse;
use std::{
    collections::VecDeque,
    io::Write,
    io::{BufReader, Read},
    sync::Arc,
    sync::Mutex,
};
pub struct Indexer {}

type BodyQueue = Arc<Mutex<VecDeque<([u8; 70], Box<dyn Read + Send>)>>>;
type ChangeIDQueue = Arc<Mutex<VecDeque<ChangeID>>>;

impl Indexer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pending_bodies = Arc::new(Mutex::new(VecDeque::new()));
        let pending_change_ids = Arc::new(Mutex::new(VecDeque::new()));

        let first_change_id = RiverClient::fetch_latest_change_id()?;
        println!("Fetched latest change id: {}", first_change_id);

        let pending_change_ids = pending_change_ids.clone();
        let pending_bodies = pending_bodies.clone();
        let pending_bodies2 = pending_bodies.clone();

        pending_change_ids
            .lock()
            .unwrap()
            .push_back(ChangeID::from_str(&first_change_id)?);

        let fetcher_handle = std::thread::spawn(move || {
            let mut ratelimit = ratelimit::Builder::new()
                .capacity(2)
                .quantum(2)
                .interval(std::time::Duration::from_millis(1_000))
                .build();

            loop {
                ratelimit.wait();

                let next_change_id = pending_change_ids.lock().unwrap().pop_front().unwrap();

                // Fetch the url...
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

        let worker_handles = (0..1)
            .map(|worker_id| {
                let pending_bodies = pending_bodies2.clone();
                std::thread::spawn(move || loop {
                    let mut lock = pending_bodies.lock().unwrap();

                    if let Some(next) = lock.pop_front() {
                        let (next_id_buffer, mut reader) = next;

                        let start = std::time::Instant::now();
                        let mut buffer = Vec::new();
                        buffer.write_all(&next_id_buffer).unwrap();
                        reader.read_to_end(&mut buffer).unwrap();

                        let deserialized =
                            serde_json::from_slice::<StashTabResponse>(&buffer).unwrap();
                        println!(
                            "Time elapsed - worker {}: {}ms -> Found {} tabs",
                            worker_id,
                            start.elapsed().as_millis(),
                            deserialized.stashes.len()
                        );
                    } else {
                        drop(lock);
                        println!("Worker #{} waiting for work", worker_id);
                        std::thread::sleep(std::time::Duration::from_millis(1_000));
                    }
                })
            })
            .collect::<Vec<_>>();

        fetcher_handle.join().expect("Fetcher thread paniced! :O");

        Ok(())
    }
}

// #[derive(Clone)]
// struct Fetcher {
//     pending_change_ids: Arc<Mutex<VecDeque<ChangeID>>>,
//     pending_bodies: Arc<Mutex<VecDeque<([u8; 70], Box<dyn Read + Send>)>>>,
// }

// impl Fetcher {
//     fn new(
//         pending_change_ids: Arc<Mutex<VecDeque<ChangeID>>>,
//         pending_bodies: Arc<Mutex<VecDeque<([u8; 70], Box<dyn Read + Send>)>>>,
//     ) -> Self {
//         Self {
//             pending_bodies,
//             pending_change_ids,
//         }
//     }
// }

// struct Worker {}
