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

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("No database url set");
    let persistence = persistence::PgDb::new(&database_url);

    let change_id = match persistence.get_last_read_change_id().ok() {
        Some(id) => {
            println!("Proceed indexing after {:?}", id);
            id
        }
        None => {
            let default_change_id =
                env::var("DEFAULT_CHANGE_ID").expect("No default change id set");
            eprintln!(
                "Could not find last change id -> Use default {}",
                default_change_id
            );
            default_change_id
        }
    };

    let mut indexer = Indexer::new(&persistence);
    indexer.start(change_id);
}
