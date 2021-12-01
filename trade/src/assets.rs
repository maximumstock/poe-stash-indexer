use std::{collections::HashMap, fs::File, io::BufWriter};

use futures::future::TryFutureExt;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

fn sort_alphabetically<T: Serialize, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}

#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AssetIndex {
    #[serde(serialize_with = "sort_alphabetically")]
    long_short_idx: HashMap<String, String>,
    #[serde(serialize_with = "sort_alphabetically")]
    short_long_idx: HashMap<String, String>,
}

const ASSET_INDEX_FILE_PATH: &str = "trade/asset_index.json";

impl AssetIndex {
    pub fn new() -> Self {
        Self {
            long_short_idx: Default::default(),
            short_long_idx: Default::default(),
        }
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let asset_response = self.reload().or_else(|_| self.fetch()).await?;

        for category in asset_response.result {
            for item in category.entries {
                self.long_short_idx
                    .insert(item.text.clone(), item.id.clone());
                self.short_long_idx.insert(item.id, item.text);
            }
        }

        self.persist().unwrap();
        Ok(())
    }

    fn persist(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(ASSET_INDEX_FILE_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    async fn reload(&self) -> Result<AssetResponse, Box<dyn std::error::Error>> {
        let reader = tokio::fs::read_to_string(ASSET_INDEX_FILE_PATH).await?;
        let asset_response = serde_json::from_str(&reader)?;
        Ok(asset_response)
    }

    async fn fetch(&self) -> Result<AssetResponse, Box<dyn std::error::Error>> {
        let mut request = reqwest::Request::new(
            reqwest::Method::GET,
            "https://www.pathofexile.com/api/trade/data/static".parse()?,
        );
        request.headers_mut().insert("user-agent", HeaderValue::from_str("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36").unwrap());
        let client = reqwest::Client::new();
        let asset_response = client
            .execute(request)
            .await
            .unwrap()
            .json::<AssetResponse>()
            .await?;
        Ok(asset_response)
    }

    pub fn has_item(&self, input: &str) -> bool {
        self.short_long_idx.contains_key(input) || self.long_short_idx.contains_key(input)
    }

    pub fn get_name(&self, id: &str) -> Option<&String> {
        self.short_long_idx.get(id)
    }

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
