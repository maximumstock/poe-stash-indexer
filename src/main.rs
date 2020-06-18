mod parser;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate ratelimit;

use dotenv::dotenv;
use parser::{parse_items, ItemParseResult, Offer, StashTabResponse};
use std::env;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

fn get_initial_change_id(persistence: &persistence::PgDb, default: String) -> String {
    if let Ok(id) = persistence.get_last_read_change_id() {
        println!("Proceed indexing after {:?}", id);
        id
    } else {
        eprintln!("Could not find last change id -> Default to {:?}", default);
        default
    }
}

#[derive(Debug)]
pub enum IndexerError {
    Persist,
    RateLimited,
    Deserialize,
    OutOfWork,
}

struct Indexer<'a> {
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
        let url = format!(
            "https://www.pathofexile.com/api/public-stash-tabs?id={}",
            id
        );

        let response = ureq::get(&url).call();

        if response.error() {
            return Err(IndexerError::RateLimited);
        }

        let txt = response.into_string().unwrap();

        serde_json::from_str::<StashTabResponse>(&txt).map_err(|_| IndexerError::Deserialize)
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
            .map_err(|_| IndexerError::Persist)
    }

    fn retry_change_id(&mut self, change_id: String) {
        self.next_change_ids.push_front(change_id);
    }

    fn handle_error(&mut self, error: &IndexerError, change_id: String) {
        match error {
            IndexerError::Deserialize => eprintln!("Deserialization failed..."),
            IndexerError::Persist => {
                self.retry_change_id(change_id);
                eprintln!("Persist failed...");
            }
            IndexerError::OutOfWork => eprintln!("Out of work..."),
            IndexerError::RateLimited => {
                self.retry_change_id(change_id);
                let timeout = Duration::from_secs(60);
                eprintln!("Rate limited... -> Sleeping {:?}", timeout);
                std::thread::sleep(timeout);
            }
        }
    }

    fn work(&mut self, change_id: String) -> Result<(), IndexerError> {
        self.load_river_id(&change_id)
            .and_then(|response| {
                let offers = self.parse(&response, &change_id);
                self.next_change_ids.push_front(response.next_change_id);
                Ok(offers)
            })
            .and_then(|offers| self.persist(offers))
            .and_then(|n| {
                println!("Processed stash id {:?}", change_id);
                if n > 0 {
                    println!("Persisting {:?} offers", n);
                }
                Ok(())
            })
            .or_else(|err| {
                self.handle_error(&err, change_id);
                Err(err)
            })
    }

    pub fn start(&mut self, change_id: String) {
        self.next_change_ids.push_front(change_id);
        loop {
            self.ratelimiter.wait();
            self.next_change_ids
                .pop_back()
                .ok_or(IndexerError::Deserialize)
                .and_then(|change_id| self.work(change_id))
                .unwrap_or_else(|_err| ());
        }
    }
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    let default_id = env::var("DEFAULT_CHANGE_ID").unwrap();

    let persistence = persistence::PgDb::new(&database_url);
    let change_id = get_initial_change_id(&persistence, default_id);

    let mut indexer = Indexer::new(&persistence);
    indexer.start(change_id);
}
