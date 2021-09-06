use std::{
    error::Error,
    io::{BufReader, Read},
    str::FromStr,
    string::FromUtf8Error,
    sync::mpsc::{Receiver, Sender},
};

use flate2::bufread::GzDecoder;
use ureq::Response;

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

#[derive(Debug)]
enum FetcherError {
    HttpError { status: u16 },
    Transport,
    RateLimited,
    ParseError,
}

impl std::fmt::Display for FetcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl Error for FetcherError {}

pub(crate) fn start_fetcher(
    fetcher_rx: Receiver<FetcherMessage>,
    scheduler_tx: Sender<SchedulerMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut ratelimit = ratelimiter();

        while let Ok(FetcherMessage::Task(task)) = fetcher_rx.recv() {
            ratelimit.wait();

            let start = std::time::Instant::now();
            log::debug!("Requesting {}", task.change_id);

            match process(&task) {
                Ok((decoder, change_id_buffer, next_change_id)) => {
                    log::debug!(
                        "fetcher: Took {}ms to read next id: {}",
                        start.elapsed().as_millis(),
                        next_change_id
                    );

                    if next_change_id.eq(&task.change_id) {
                        ratelimit.wait_for(2);
                    }

                    if task.reschedule_count == 0 {
                        scheduler_tx
                            .send(SchedulerMessage::Fetch(FetchTask {
                                change_id: next_change_id,
                                reschedule_count: 0,
                            }))
                            .unwrap();
                    }

                    scheduler_tx
                        .send(SchedulerMessage::Work(WorkerTask {
                            reader: Box::new(decoder),
                            fetch_partial: change_id_buffer,
                            change_id: task.change_id,
                        }))
                        .unwrap();
                }
                Err(
                    FetcherError::Transport
                    | FetcherError::HttpError { .. }
                    | FetcherError::ParseError,
                ) => {
                    reschedule_task(&scheduler_tx, task);
                }
                Err(FetcherError::RateLimited) => {
                    log::info!("fetcher: Rate limit reached");
                    scheduler_tx.send(SchedulerMessage::Stop).unwrap();
                    break;
                }
            }
        }

        log::debug!("fetcher: Shutting down");
    })
}

fn reschedule_task(scheduler_tx: &Sender<SchedulerMessage>, task: FetchTask) {
    if task.reschedule_count > 2 {
        scheduler_tx.send(SchedulerMessage::Stop).unwrap();
        return;
    }

    log::info!(
        "fetcher: Rescheduling {} (Retried {} times)",
        task.change_id,
        task.reschedule_count
    );

    scheduler_tx
        .send(SchedulerMessage::Fetch(FetchTask {
            reschedule_count: task.reschedule_count + 1,
            ..task
        }))
        .unwrap();
}

fn ratelimiter() -> ratelimit::Limiter {
    // Break down rate-limit into quantum of 1, so we never do any bursts,
    // like we would with for example 2 requests per second.
    ratelimit::Builder::new()
        .capacity(1)
        .quantum(1)
        .interval(std::time::Duration::from_millis(500))
        .build()
}

fn process(
    task: &FetchTask,
) -> Result<(GzDecoder<BufReader<impl Read + Send>>, [u8; 80], ChangeId), FetcherError> {
    fetch_chunk(task).and_then(parse_chunk)
}

fn parse_chunk(
    response: Response,
) -> Result<(GzDecoder<BufReader<impl Read + Send>>, [u8; 80], ChangeId), FetcherError> {
    let reader = response.into_reader();
    let mut decoder = GzDecoder::new(BufReader::new(reader));
    let mut next_id_buffer = [0; 80];

    match decoder.read_exact(&mut next_id_buffer) {
        Ok(_) => {
            let next_id = parse_change_id_from_bytes(&next_id_buffer)
                .map_err(|_| FetcherError::ParseError)
                .and_then(|s| ChangeId::from_str(&s).map_err(|_| FetcherError::ParseError))?;
            Ok((decoder, next_id_buffer, next_id))
        }
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
            log::error!("fetcher: UnexpectedEof: {:?}", next_id_buffer);
            Err(FetcherError::ParseError)
        }
        Err(err) => {
            log::error!("fetcher: gzip decoding failed: {}", err);
            Err(FetcherError::ParseError)
        }
    }
}

fn fetch_chunk(task: &FetchTask) -> Result<Response, FetcherError> {
    let url = format!(
        "http://www.pathofexile.com/api/public-stash-tabs?id={}",
        task.change_id
    );

    let response = ureq::request("GET", &url)
        .set("Accept-Encoding", "gzip")
        .set("Accept", "application/json")
        .call();

    response.map_err(|e| match e {
        ureq::Error::Status(status, ref response) => {
            log::error!("fetcher: HTTP error {}", status);
            log::error!("fetcher: HTTP response: {:?}", response);

            match status {
                429 => FetcherError::RateLimited,
                _ => FetcherError::HttpError { status },
            }
        }
        ureq::Error::Transport(_) => FetcherError::Transport,
    })
}

pub fn parse_change_id_from_bytes(bytes: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(
        bytes
            .split(|b| (*b as char).eq(&'"'))
            .nth(3)
            .unwrap()
            .to_vec(),
    )
}
