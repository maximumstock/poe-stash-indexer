use crate::parser::*;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::{Read, Write};

// fn main() {
//     let mut queue = std::collections::VecDeque::<String>::new();
//     queue.push_front("844329005-856873618-819256718-924606015-883951065".into());
//     let client = RiverClient {};

//     loop {
//         if queue.is_empty() {
//             std::process::exit(-1);
//         }
//         println!("Loop started");
//         let url = format!(
//             "http://www.pathofexile.com/api/public-stash-tabs?id={}",
//             queue.pop_back().unwrap()
//         );
//         let tabs = client.fetch_json(&url).unwrap();
//         println!("Found next change id: {:#}", tabs.next_change_id);
//         queue.push_back(tabs.next_change_id);
//     }
// }

#[derive(Debug, Deserialize)]
pub(crate) struct POENinjaGetStats {
    next_change_id: String,
}

#[derive(Debug)]
pub struct RiverClient {}

impl RiverClient {
    fn fetch_changes_for_id(
        change_id: &str,
    ) -> Result<StashTabResponse, Box<dyn std::error::Error>> {
        // Fetch the url...
        let start = std::time::Instant::now();
        let url = format!(
            "http://www.pathofexile.com/api/public-stash-tabs?id={}",
            change_id
        );
        let mut request = ureq::request("GET", &url);
        request.set("Accept-Encoding", "gzip");
        request.set("Accept", "application/json");
        let response = request.call();
        let reader = response.into_reader();
        println!("Time elapsed - Fetching: {}ms", start.elapsed().as_millis());

        let mut reader = GzDecoder::new(reader);
        let start = std::time::Instant::now();
        let mut next_id_buffer = [0; 70];
        reader.read_exact(&mut next_id_buffer)?;
        let next_id_raw = next_id_buffer
            .iter()
            .skip(19)
            .take(49)
            .cloned()
            .collect::<Vec<u8>>();
        let next_id = String::from_utf8(next_id_raw).unwrap();
        println!(
            "Time elapsed - reading next_id: {}ms - next: {}",
            start.elapsed().as_millis(),
            next_id
        );
        let start = std::time::Instant::now();
        let mut buffer = Vec::new();
        buffer.write_all(change_id.as_bytes())?;
        reader.read_to_end(&mut buffer).unwrap();
        println!(
            "Time elapsed - reading body: {}ms, length: {}",
            start.elapsed().as_millis(),
            buffer.len()
        );

        let start = std::time::Instant::now();
        let deserialized = serde_json::from_slice(&buffer)?;
        println!(
            "Time elapsed - deserialization: {}ms",
            start.elapsed().as_millis()
        );
        Ok(deserialized)
    }

    pub fn fetch_latest_change_id() -> Result<String, Box<dyn std::error::Error>> {
        let response = ureq::get("https://poe.ninja/api/Data/GetStats").call();
        let stats: POENinjaGetStats = serde_json::from_reader(response.into_reader())?;
        Ok(stats.next_change_id)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_fetch_latest_change_id() {
        let latest_change_id = super::RiverClient::fetch_latest_change_id();
        assert!(latest_change_id.is_ok());
        assert_eq!(latest_change_id.unwrap().len(), 49);
    }
}
