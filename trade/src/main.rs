use source::ExampleStream;
use store::Store;

use crate::assets::AssetIndex;

/// TODO
/// - RabbitMQ client that produces a stream of `StashRecord`s
/// - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// - a web API that mimics pathofexile.com/trade API
/// - will need state snapshots + restoration down the road
/// - filter currency items from `StashRecord`
///   - need asset mapping from pathofexile.com/trade
mod store;

fn main() {
    let mut asset_index = AssetIndex::new();
    asset_index.init();

    let example = ExampleStream::new("./data.json");
    let mut store = Store::new("Scourge");

    for stash_record in example {
        store.ingest_stash(stash_record, &asset_index);
    }

    println!("Store has {:#?} offers", store.size());
}

mod assets {
    use std::{
        collections::HashMap,
        fs::File,
        io::{BufReader, BufWriter},
    };

    use reqwest::header::HeaderValue;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub struct AssetIndex {
        long_short_idx: HashMap<String, String>,
        short_long_idx: HashMap<String, String>,
    }

    impl AssetIndex {
        pub fn new() -> Self {
            Self {
                long_short_idx: Default::default(),
                short_long_idx: Default::default(),
            }
        }

        pub fn init(&mut self) {
            let asset_response = self.reload().or_else(|_| self.fetch()).unwrap();

            for category in asset_response.result {
                for item in category.entries {
                    self.long_short_idx
                        .insert(item.text.clone(), item.id.clone());
                    self.short_long_idx.insert(item.text, item.id);
                }
            }

            self.persist().unwrap();
        }

        fn persist(&self) -> Result<(), Box<dyn std::error::Error>> {
            let file = File::create("asset_index.json")?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &self)?;
            Ok(())
        }

        fn reload(&self) -> Result<AssetResponse, Box<dyn std::error::Error>> {
            let file = File::open("asset_index.json")?;
            let reader = BufReader::new(file);
            let asset_response = serde_json::from_reader(reader)?;
            Ok(asset_response)
        }

        fn fetch(&self) -> Result<AssetResponse, Box<dyn std::error::Error>> {
            let mut request = reqwest::blocking::Request::new(
                reqwest::Method::GET,
                "https://www.pathofexile.com/api/trade/data/static".parse()?,
            );
            request.headers_mut().insert("user-agent", HeaderValue::from_str("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36").unwrap());
            let client = reqwest::blocking::Client::new();
            let asset_response = client.execute(request).unwrap().json::<AssetResponse>()?;
            Ok(asset_response)
        }

        pub fn has_item(&self, input: &str) -> bool {
            self.short_long_idx.contains_key(input) || self.long_short_idx.contains_key(input)
        }

        // pub fn get_name(&self, id: &str) -> Option<&String> {
        //     self.short_long_idx.get(id)
        // }

        // pub fn get_id(&self, name: &str) -> Option<&String> {
        //     self.long_short_idx.get(name)
        // }
    }

    #[derive(Debug, Deserialize)]
    struct AssetResponse {
        result: Vec<AssetCategory>,
    }

    #[derive(Debug, Deserialize)]
    struct AssetCategory {
        id: String,
        entries: Vec<AssetItem>,
    }

    #[derive(Debug, Deserialize)]
    struct AssetItem {
        id: String,
        text: String,
    }
}

mod source {
    use std::{fs::File, io::BufReader, path::Path};

    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct StashRecord {
        pub stash_id: String,
        pub league: String,
        pub account_name: String,
        pub items: Vec<Item>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Item {
        pub id: String,
        pub type_line: String,
        pub note: Option<String>,
        pub stack_size: Option<u32>,
    }

    pub struct ExampleStream {
        stash_records: Vec<StashRecord>,
    }

    impl IntoIterator for ExampleStream {
        type Item = StashRecord;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.stash_records.into_iter()
        }
    }

    impl ExampleStream {
        pub fn new<T: AsRef<Path>>(file_path: T) -> Self {
            let reader = BufReader::new(File::open(file_path).unwrap());
            let stash_records = serde_json::de::from_reader::<_, Vec<StashRecord>>(reader).unwrap();

            Self { stash_records }
        }
    }
}
