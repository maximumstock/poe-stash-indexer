use crate::parser::{parse_items, ItemParseResult, Offer, StashTabResponse};
use crate::persistence;
use std::time::Duration;

#[derive(Debug)]
pub enum IndexerError {
    Persist(String),
    RateLimited,
    Deserialize(String),
    NonUtf8Response(String),
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

    fn load_river_id(&self, id: &str) -> Result<StashTabResponse, IndexerError> {
        let url = format!("http://www.pathofexile.com/api/public-stash-tabs?id={}", id);
        let response = minreq::get(&url).send().unwrap();

        if response.status_code.eq(&429) {
            return Err(IndexerError::RateLimited);
        }

        response
            .as_str()
            .map_err(|e| IndexerError::NonUtf8Response(e.to_string()))
            .and_then(|txt| {
                serde_json::from_str::<StashTabResponse>(&txt)
                    .map_err(|e| IndexerError::Deserialize(e.to_string()))
            })
    }

    fn parse(&mut self, response: &StashTabResponse, id: &str) -> Vec<Offer> {
        let mut offers = vec![];
        for result in parse_items(&response, id) {
            match result {
                ItemParseResult::Success(item_log) => offers.push(item_log),
                ItemParseResult::Error(e) => {
                    println!("Error: {:?}", e);
                }
                ItemParseResult::Empty => {}
            }
        }
        offers
    }

    pub fn persist(&mut self, offers: Vec<Offer>) -> Result<usize, IndexerError> {
        self.persistence
            .save_offers(&offers)
            .map_err(|e| IndexerError::Persist(e.to_string()))
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

    fn work(&mut self, change_id: &str) -> Result<(), IndexerError> {
        self.load_river_id(&change_id)
            .and_then(|response| {
                let offers = self.parse(&response, &change_id);
                self.next_change_ids.push_front(response.next_change_id);
                Ok(offers)
            })
            .and_then(|offers| self.persist(offers))
            .and_then(|_| Ok(()))
    }

    pub fn start(&mut self, change_id: String) {
        self.next_change_ids.push_front(change_id);
        loop {
            self.ratelimiter.wait();

            if let Some(next_id) = self.next_change_ids.pop_back() {
                let result = self.work(&next_id);
                match result {
                    Ok(_) => println!("Processed stash id {:?}", next_id),
                    Err(e) => self.handle_error(&e, next_id),
                }
            } else {
                std::process::exit(-1);
            }
        }
    }
}
