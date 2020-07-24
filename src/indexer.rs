use crate::parser::{parse_items, ItemParseResult, Offer, StashTabResponse};
use crate::persistence;
use minreq::Response;
use std::time::Duration;

#[derive(Debug)]
pub enum IndexerError {
    Persist(String),
    RateLimited,
    Deserialize(String),
    NonUtf8Response(String),
}

#[derive(Debug)]
pub struct IndexerResult<T> {
    data: T,
    s_fetch: Option<Duration>,
    s_deserialize: Option<Duration>,
    s_parse: Option<Duration>,
    s_persist: Option<Duration>,
}

impl<T> IndexerResult<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            s_deserialize: None,
            s_fetch: None,
            s_parse: None,
            s_persist: None,
        }
    }

    pub fn with_fetch(mut self, duration: Option<Duration>) -> Self {
        self.s_fetch = duration;
        self
    }

    pub fn with_parse(mut self, duration: Option<Duration>) -> Self {
        self.s_parse = duration;
        self
    }

    pub fn with_deserialize(mut self, duration: Option<Duration>) -> Self {
        self.s_deserialize = duration;
        self
    }

    pub fn with_persist(mut self, duration: Option<Duration>) -> Self {
        self.s_persist = duration;
        self
    }
}

pub struct Indexer<'a> {
    persistence: &'a persistence::PgDb,
    next_change_ids: std::collections::VecDeque<String>,
    ratelimiter: ratelimit::Limiter,
}

impl<'a> Indexer<'a> {
    pub fn new(persistence: &'a persistence::PgDb) -> Self {
        Indexer {
            persistence,
            next_change_ids: std::collections::VecDeque::new(),
            ratelimiter: ratelimit::Builder::new()
                .capacity(2)
                .interval(Duration::from_secs(2))
                .quantum(1)
                .build(),
        }
    }

    fn fetch_river_id(&self, id: &str) -> Result<IndexerResult<Response>, IndexerError> {
        let start = std::time::Instant::now();

        let url = format!("http://www.pathofexile.com/api/public-stash-tabs?id={}", id);
        let response = minreq::get(&url).send().unwrap();

        if response.status_code.eq(&429) {
            return Err(IndexerError::RateLimited);
        }

        Ok(IndexerResult::new(response).with_fetch(Some(start.elapsed())))
    }

    fn deserialize(
        &self,
        response: IndexerResult<Response>,
    ) -> Result<IndexerResult<StashTabResponse>, IndexerError> {
        let start = std::time::Instant::now();

        response
            .data
            .as_str()
            .map_err(|e| IndexerError::NonUtf8Response(e.to_string()))
            .and_then(|txt| {
                serde_json::from_str::<StashTabResponse>(&txt)
                    .map_err(|e| IndexerError::Deserialize(e.to_string()))
            })
            .map(|data| {
                IndexerResult::new(data)
                    .with_fetch(response.s_fetch)
                    .with_deserialize(Some(start.elapsed()))
            })
    }

    fn parse(
        &mut self,
        result: IndexerResult<StashTabResponse>,
        id: &str,
    ) -> Result<IndexerResult<Vec<Offer>>, IndexerError> {
        let start = std::time::Instant::now();
        let mut offers = vec![];
        for result in parse_items(&result.data, id) {
            match result {
                ItemParseResult::Success(item_log) => offers.push(item_log),
                ItemParseResult::Error(e) => {
                    println!("Error: {:?}", e);
                }
                ItemParseResult::Empty => {}
            }
        }
        Ok(IndexerResult::new(offers)
            .with_fetch(result.s_fetch)
            .with_deserialize(result.s_deserialize)
            .with_parse(Some(start.elapsed())))
    }

    pub fn persist(
        &mut self,
        result: IndexerResult<Vec<Offer>>,
    ) -> Result<IndexerResult<usize>, IndexerError> {
        let start = std::time::Instant::now();
        self.persistence
            .save_offers(&result.data)
            .map_err(|e| IndexerError::Persist(e.to_string()))
            .map(|x| {
                IndexerResult::new(x)
                    .with_fetch(result.s_fetch)
                    .with_deserialize(result.s_deserialize)
                    .with_parse(result.s_parse)
                    .with_persist(Some(start.elapsed()))
            })
    }

    fn retry_change_id(&mut self, change_id: String) {
        self.next_change_ids.push_front(change_id);
    }

    fn handle_error(&mut self, error: &IndexerError, change_id: String) {
        self.retry_change_id(change_id);
        match error {
            IndexerError::NonUtf8Response(e) => eprintln!("Encountered non-utf8 response: {}", e),
            IndexerError::Deserialize(e) => eprintln!("Deserialization failed: {}", e),
            IndexerError::Persist(e) => eprintln!("Persist failed: {}", e),
            IndexerError::RateLimited => {
                let timeout = Duration::from_secs(60);
                eprintln!("Rate limited... -> Sleeping {:?}", timeout);
                std::thread::sleep(timeout);
            }
        }
    }

    fn work(&mut self, change_id: &str) -> Result<IndexerResult<usize>, IndexerError> {
        self.fetch_river_id(&change_id)
            .and_then(|result| self.deserialize(result))
            .and_then(|result| {
                self.next_change_ids
                    .push_front(result.data.next_change_id.clone());
                self.parse(result, &change_id)
            })
            .and_then(|offers| self.persist(offers))
    }

    pub fn start(&mut self, change_id: String) {
        self.next_change_ids.push_front(change_id);
        loop {
            self.ratelimiter.wait();

            if let Some(next_id) = self.next_change_ids.pop_back() {
                match self.work(&next_id) {
                    Ok(result) => {
                        println!(
                            "Processed stash id {:?}\n\tFetch: {:?}ms\tDeserialize: {:?}ms\tParse: {:?}ms\tPersist: {:?}ms",
                            next_id,
                            result.s_fetch.unwrap().as_millis(),
                            result.s_deserialize.unwrap().as_millis(),
                            result.s_parse.unwrap().as_millis(),
                            result.s_persist.unwrap().as_millis(),
                        )
                    }
                    Err(e) => self.handle_error(&e, next_id),
                }
            } else {
                std::process::exit(-1);
            }
        }
    }
}
