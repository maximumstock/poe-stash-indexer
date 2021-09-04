use std::{
    io::{BufReader, Read},
    str::FromStr,
    string::FromUtf8Error,
    sync::mpsc::{Receiver, Sender},
};

use flate2::bufread::GzDecoder;
use ureq::Error;

use crate::{common::ChangeId, sync::worker::WorkerTask};

use super::scheduler::SchedulerMessage;

pub(crate) enum FetcherMessage {
    Task(FetchTask),
    Stop,
}

#[derive(Debug, Clone)]
pub(crate) struct FetchTask {
    change_id: ChangeId,
    reschedule_count: u32,
}

impl FetchTask {
    pub(crate) fn new(change_id: ChangeId) -> Self {
        Self {
            change_id,
            reschedule_count: 0,
        }
    }
}

pub(crate) fn start_fetcher(
    fetcher_rx: Receiver<FetcherMessage>,
    scheduler_tx: Sender<SchedulerMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // Break down rate-limit into quantum of 1, so we never do any bursts,
        // like we would with for example 2 requests per second.
        let mut ratelimit = ratelimit::Builder::new()
            .capacity(1)
            .quantum(1)
            .interval(std::time::Duration::from_millis(500))
            .build();

        while let Ok(FetcherMessage::Task(task)) = fetcher_rx.recv() {
            ratelimit.wait();

            let start = std::time::Instant::now();
            let url = format!(
                "http://www.pathofexile.com/api/public-stash-tabs?id={}",
                task.change_id
            );

            log::debug!("Requesting {}", task.change_id);
            let request = ureq::request("GET", &url)
                .set("Accept-Encoding", "gzip")
                .set("Accept", "application/json");
            let response = request.call();

            if let Err(Error::Status(status, ref response)) = response {
                log::error!("fetcher: HTTP error {}", status);
                log::error!("fetcher: HTTP response: {:?}", response);

                // match reschedule(shared_state.clone(), change_id_request) {
                //     Ok(_) => continue,
                //     Err(_) => break,
                // }
            }

            let reader = response.unwrap().into_reader();
            let mut decoder = GzDecoder::new(BufReader::new(reader));
            let mut next_id_buffer = [0; 80];

            match decoder.read_exact(&mut next_id_buffer) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    log::error!("UnexpectedEof: {:?}", next_id_buffer);
                    continue;
                }
                Err(err) => {
                    log::error!("fetcher: gzip decoding failed: {}", err);

                    // match reschedule(shared_state.clone(), change_id_request.clone()) {
                    //     Ok(_) => continue,
                    //     Err(_) => break,
                    // }
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

            if next_change_id.eq(&task.change_id) {
                ratelimit.wait_for(2);
            }

            scheduler_tx
                .send(SchedulerMessage::Fetch(FetchTask {
                    change_id: next_change_id,
                    reschedule_count: 0,
                }))
                .unwrap();
            scheduler_tx
                .send(SchedulerMessage::Work(WorkerTask {
                    reader: Box::new(decoder),
                    fetch_partial: next_id_buffer,
                    change_id: task.change_id,
                }))
                .unwrap();
        }

        log::debug!("Shut down fetcher");
    })
}

// fn reschedule(shared_state: SharedState, request: ChangeIdRequest) -> Result<(), ()> {
//     if request.1 > 2 {
//         log::error!("Retried too many times...shutting down");
//         return Err(());
//     }

//     let new_request = (request.0, request.1 + 1);
//     log::info!(
//         "Rescheduling {} (Retried {} times)",
//         new_request.0,
//         request.1
//     );
//     shared_state
//         .lock()
//         .unwrap()
//         .fetcher_queue
//         .push_back(new_request);

//     Ok(())
// }

pub fn parse_change_id_from_bytes(bytes: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(
        bytes
            .split(|b| (*b as char).eq(&'"'))
            .nth(3)
            .unwrap()
            .to_vec(),
    )
}
