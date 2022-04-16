use std::collections::HashMap;

use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let asset_response = self.fetch().await?;

        for category in asset_response.result {
            for item in category.entries {
                self.long_short_idx
                    .insert(item.text.clone(), item.id.clone());
                self.short_long_idx.insert(item.id, item.text);
            }
        }

        Ok(())
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

    pub fn get_name(&self, id: &str) -> Option<&String> {
        self.short_long_idx.get(id)
    }
}

#[derive(Debug, Deserialize)]
struct AssetResponse {
    result: Vec<AssetCategory>,
}

#[derive(Debug, Deserialize)]
struct AssetCategory {
    entries: Vec<AssetItem>,
}

#[derive(Debug, Deserialize)]
struct AssetItem {
    id: String,
    text: String,
}
