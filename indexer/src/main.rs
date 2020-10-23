mod parser;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use lib::Indexer;
// use dotenv::dotenv;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_timed();
    // dotenv().ok();
    // // let database_url = env::var("DATABASE_URL").expect("No database url set");
    // // let persistence = persistence::PgDb::new(&database_url);
    // let persistence = persistence::CSVLog::new("log.csv");

    // let latest_change_id = client::RiverClient::fetch_latest_change_id()?;
    // let client = client::RiverClient {};
    // let mut indexer = Indexer::new(&persistence, &client);
    // indexer.start(latest_change_id);

    let indexer = Indexer::new();
    let rx = indexer.start_with_latest()?;

    while let Ok(stash_tab_response) = rx.recv() {
        log::info!("Found {} stash tabs", stash_tab_response.stashes.len());
    }

    Ok(())
}
