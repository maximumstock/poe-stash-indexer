extern crate log;
extern crate pretty_env_logger;

mod db;
mod differ;
mod processing;
pub mod stash;
mod store;

use std::sync::Arc;

use db::StashRecordIterator;
use dotenv::dotenv;
use processing::{aggregation_consumer, flat_consumer, SharedState};
use sqlx::postgres::PgPoolOptions;

fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    pretty_env_logger::init();

    let database_url =
        std::env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable");
    let league = std::env::var("LEAGUE").expect("Missing LEAGUE environment variable");
    let use_flat_consumer = std::env::args().any(|x| x.eq("--flat"));

    let shared_state = SharedState::default();
    let shared_state2 = Arc::clone(&shared_state);

    let producer =
        std::thread::spawn(move || producer(shared_state, database_url.as_ref(), league.as_ref()));

    let consumer = if use_flat_consumer {
        log::info!("Using aggregation consumer");
        std::thread::spawn(|| aggregation_consumer(shared_state2))
    } else {
        log::info!("Using flat consumer");
        std::thread::spawn(|| flat_consumer(shared_state2))
    };

    producer.join().unwrap();
    consumer.join().unwrap();

    Ok(())
}

fn producer(shared_state: SharedState, database_url: &str, league: &str) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let pool = runtime
        .block_on(async {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
        })
        .unwrap();

    let mut iterator = StashRecordIterator::new(&pool, &runtime, 10000, league);

    while let Some(next) = iterator.next_chunk() {
        let (lock, cvar) = &*shared_state;
        let queue = &mut lock.lock().unwrap().queue;
        queue.push_back(next);

        if queue.len() > 3 {
            cvar.notify_one();
        }
    }
}
