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
