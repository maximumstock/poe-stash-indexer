mod parser;

use parser::{parse_items, ItemParseError, ItemParseResult, Offer, StashTabResponse};
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate lazy_static;

fn load_river_id(id: &str) -> Result<StashTabResponse, ItemParseError> {
    let url = format!(
        "https://www.pathofexile.com/api/public-stash-tabs?id={}",
        id
    );

    ureq::get(&url)
        .call()
        .into_string()
        .map(|x| serde_json::from_str::<StashTabResponse>(&x).unwrap())
        .map_err(|_| ItemParseError::Fetch)
}

fn do_loop(id: &str, offers: &mut Vec<Offer>) -> Option<String> {
    match load_river_id(&id) {
        Ok(response) => {
            for result in parse_items(&response, id) {
                match result {
                    ItemParseResult::Success(item_log) => offers.push(item_log),
                    ItemParseResult::Error(e) => {
                        println!("Error: {:?}", e);
                    }
                    ItemParseResult::Empty => {}
                }
            }
            println!("Processed stash id {:?}", id);
            Some(response.next_change_id)
        }
        Err(e) => {
            println!("Error when fetching id {}: Error: {:?}", id, e);
            None
        }
    }
}

fn save_item_logs(offers: &Vec<Offer>) -> () {
    let mut file = File::create("db.json").unwrap();
    let output = serde_json::to_string_pretty(offers).unwrap();
    file.write(output.as_ref()).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut item_logs = vec![];

    let default_id = String::from("717821295-732074652-698784848-789924768-78833560");
    let mut id = args.get(1).unwrap_or(&default_id).clone();

    let mut limit = 1000;
    loop {
        if item_logs.len() >= limit {
            println!("Reached limit ({:?}): Saving db.json", limit);
            save_item_logs(&item_logs);
            limit = limit + limit;
        }
        match do_loop(&id, &mut item_logs) {
            Some(new_id) => id = new_id,
            None => {}
        }
    }
}
