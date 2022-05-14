use std::{
    error::Error,
    io::{BufReader, Read},
    str::FromStr,
    string::FromUtf8Error,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use ureq::Response;

use crate::{common::ChangeId, sync::worker::WorkerTask};

use super::scheduler::SchedulerMessage;

const DEFAULT_RATE_LIMIT_TIMER: u64 = 60;

pub(crate) enum FetcherMessage {
    Task(FetchTask),
    Stop,
}

#[derive(Debug, Clone)]
pub(crate) struct FetchTask {
    change_id: ChangeId,
}

impl FetchTask {
    pub(crate) fn new(change_id: ChangeId) -> Self {
        Self { change_id }
    }

    pub(crate) fn retry(self) -> Option<Self> {
        Some(FetchTask { ..self })
    }
}

#[derive(Debug)]
enum FetcherError {
    #[allow(dead_code)]
    HttpError {
        status: u16,
    },
    Transport,
    ServiceUnavailable,
    RateLimited(Duration),
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

            match fetch_chunk(&task).and_then(parse_chunk) {
                Ok((decoder, change_id_buffer, next_change_id)) => {
                    log::debug!(
                        "fetcher: Took {}ms to read next id: {}",
                        start.elapsed().as_millis(),
                        next_change_id
                    );

                    scheduler_tx
                        .send(SchedulerMessage::Fetch(FetchTask {
                            change_id: next_change_id.clone(),
                        }))
                        .unwrap();

                    scheduler_tx
                        .send(SchedulerMessage::Work(WorkerTask {
                            reader: Box::new(decoder),
                            fetch_partial: change_id_buffer,
                            change_id: task.change_id.clone(),
                        }))
                        .unwrap();

                    if next_change_id.eq(&task.change_id) {
                        ratelimit.wait_for(4);
                    }
                }
                Err(
                    e @ (FetcherError::HttpError { .. }
                    | FetcherError::ParseError
                    | FetcherError::Transport),
                ) => {
                    log::error!("fetcher: Encountered error {}", e);
                    reschedule_task(&scheduler_tx, task);
                }
                Err(FetcherError::ServiceUnavailable) => {
                    log::error!("fetcher: Service Unavailable - Retrying in 60s");
                    scheduler_tx
                        .send(SchedulerMessage::RateLimited(Duration::from_secs(60)))
                        .unwrap();
                    reschedule_task(&scheduler_tx, task);
                }
                Err(FetcherError::RateLimited(timer)) => {
                    log::info!("fetcher: Rate limit reached");
                    scheduler_tx
                        .send(SchedulerMessage::RateLimited(timer))
                        .unwrap();
                    reschedule_task(&scheduler_tx, task);
                }
            }
        }

        log::debug!("fetcher: Shutting down");
    })
}

fn reschedule_task(scheduler_tx: &Sender<SchedulerMessage>, task: FetchTask) {
    match task.retry() {
        Some(t) => {
            log::info!("fetcher: Rescheduling {}", t.change_id,);
            scheduler_tx.send(SchedulerMessage::Fetch(t)).unwrap();
        }
        None => {
            scheduler_tx.send(SchedulerMessage::Stop).unwrap();
        }
    }
}

fn ratelimiter() -> ratelimit::Limiter {
    // Break down rate-limit into quantum of 1, so we never do any bursts,
    // like we would with for example 2 requests per second.
    ratelimit::Builder::new()
        .capacity(1)
        .quantum(1)
        .interval(std::time::Duration::from_millis(600))
        .build()
}

fn parse_chunk(
    response: Response,
) -> Result<(BufReader<impl Read + Send>, [u8; 80], ChangeId), FetcherError> {
    let reader = response.into_reader();
    let mut decoder = BufReader::new(reader);
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
        .set("Accept", "application/json")
        .call();

    response.map_err(|e| match e {
        ureq::Error::Status(status, ref response) => {
            log::error!("fetcher: HTTP error {}", status);
            log::error!("fetcher: HTTP response: {:?}", response);

            match status {
                429 => {
                    let wait_time = parse_rate_limit_timer(response.header("x-rate-limit-ip"));
                    FetcherError::RateLimited(wait_time)
                }
                503 => FetcherError::ServiceUnavailable,
                _ => FetcherError::HttpError { status },
            }
        }
        ureq::Error::Transport(_) => FetcherError::Transport,
    })
}

pub fn parse_rate_limit_timer(input: Option<&str>) -> Duration {
    let seconds = input
        .and_then(|v| v.split(':').last())
        .map(|s| {
            if s.ne("60") {
                log::warn!("Expected x-rate-limit-ip to be 60 seconds");
            }
            s.parse().unwrap_or(DEFAULT_RATE_LIMIT_TIMER)
        })
        .unwrap_or(DEFAULT_RATE_LIMIT_TIMER);

    Duration::from_secs(seconds)
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_rate_limit_timer;

    #[test]
    fn test_parse_rate_limit_timer() {
        assert_eq!(parse_rate_limit_timer(None), Duration::from_secs(60));
        assert_eq!(
            parse_rate_limit_timer(Some("something")),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_rate_limit_timer(Some("_:_:abc")),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_rate_limit_timer(Some("_:_:120")),
            Duration::from_secs(120)
        );
    }
}
