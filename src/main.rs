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

fn get_initial_change_id(persistence: &persistence::PgDb) -> Option<String> {
    if let Ok(id) = persistence.get_last_read_change_id() {
        println!("Proceed indexing after {:?}", id);
        Some(id)
    } else {
        eprintln!("Could not find last change id -> Use default");
        None
    }
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    let default_id = env::var("DEFAULT_CHANGE_ID").unwrap();

    let persistence = persistence::PgDb::new(&database_url);
    let change_id = get_initial_change_id(&persistence).unwrap_or(default_id);

    let mut indexer = Indexer::new(&persistence);
    indexer.start(change_id);
}
