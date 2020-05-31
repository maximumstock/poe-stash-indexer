mod parser;

use parser::{parse_items, ItemLog, ItemParseError, ItemParseResult, StashTabResponse};

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

fn do_loop(id: &str, item_logs: &mut Vec<ItemLog>) -> Option<String> {
    match load_river_id(&id) {
        Ok(response) => {
            for result in parse_items(&response) {
                match result {
                    ItemParseResult::Success(item_log) => item_logs.push(item_log),
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

fn main() {
    let mut item_logs = vec![];
    let mut id = String::from("717821295-732074652-698784848-789924768-78833560");
    loop {
        match do_loop(&id, &mut item_logs) {
            Some(new_id) => id = new_id,
            None => {}
        }
    }
}
