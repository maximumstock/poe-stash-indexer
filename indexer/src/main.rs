mod parser;
mod persistence;
mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use dotenv::dotenv;
use lib::Indexer;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("No database url set");
    let persistence = persistence::PgDb::new(&database_url);

    let indexer = Indexer::new();
    let rx = indexer.start_with_latest()?;

    while let Ok(stash_tab_response) = rx.recv() {
        log::info!("Found {} stash tabs", stash_tab_response.stashes.len());
    }

    Ok(())
}
