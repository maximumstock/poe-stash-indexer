use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = SampleImporter::from_file("./sample_raw.csv");
    for record in stash_records {
        println!("{:?}", record);
    }

    Ok(())
}

struct SampleImporter;

impl SampleImporter {
    fn from_file(file_path: &str) -> Vec<StashRecord> {
        let mut reader = ReaderBuilder::new()
            .from_path(file_path)
            .expect("File not found");

        reader
            .deserialize::<HashMap<String, String>>()
            .into_iter()
            .map(|record| {
                let record = record.unwrap();
                StashRecord {
                    stash_id: record.get("stash_id").unwrap().clone(),
                    stash_type: record.get("stash_type").unwrap().clone(),
                    account_name: record.get("account_name").unwrap().clone(),
                    stash_name: record.get("stash_name").unwrap().clone(),
                    league: record.get("league").unwrap().clone(),
                    items: serde_json::from_str::<Vec<Item>>(&record.get("items").unwrap())
                        .unwrap(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct StashRecord {
    stash_id: String,
    stash_type: String,
    items: Vec<Item>,
    account_name: String,
    stash_name: String,
    league: String,
}

#[derive(Debug, Deserialize)]
struct Item {
    id: String,
}
