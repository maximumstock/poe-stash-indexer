use source::ExampleStream;
use store::Store;

/// TODO
/// - RabbitMQ client that produces a stream of `StashRecord`s
/// - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// - a web API that mimics pathofexile.com/trade API
/// - will need state snapshots + restoration down the road
mod store;

fn main() {
    let example = ExampleStream::new("./data.json");
    let mut store = Store::new("Scourge");

    for stash_record in example {
        store.ingest_stash(stash_record);
    }

    println!("{:#?}", &store);
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
