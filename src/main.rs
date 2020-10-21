use indexer2::Indexer;

mod client;
// mod indexer;
mod indexer2;
mod parser;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate ratelimit;

// use dotenv::dotenv;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // dotenv().ok();
    // // let database_url = env::var("DATABASE_URL").expect("No database url set");
    // // let persistence = persistence::PgDb::new(&database_url);
    // let persistence = persistence::CSVLog::new("log.csv");

    // let latest_change_id = client::RiverClient::fetch_latest_change_id()?;
    // let client = client::RiverClient {};
    // let mut indexer = Indexer::new(&persistence, &client);
    // indexer.start(latest_change_id);

    let indexer = Indexer::new();
    indexer.start()?;

    Ok(())
}
