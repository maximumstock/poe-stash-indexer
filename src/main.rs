mod client;
mod indexer;
mod parser;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate ratelimit;

use dotenv::dotenv;
use indexer::Indexer;
use std::env;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // let database_url = env::var("DATABASE_URL").expect("No database url set");
    // let persistence = persistence::PgDb::new(&database_url);
    let persistence = persistence::CSVLog::new("log.csv");

    let latest_change_id = client::fetch_latest_change_id()?;
    let mut indexer = Indexer::new(&persistence);
    indexer.start(latest_change_id);

    Ok(())
}
