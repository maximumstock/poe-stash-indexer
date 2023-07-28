use crate::common::ChangeId;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct PoeNinjaGetStats {
    next_change_id: String,
}

#[derive(Debug)]
pub struct PoeNinjaClient;

impl PoeNinjaClient {
    pub async fn fetch_latest_change_id_async() -> Result<ChangeId, Box<dyn std::error::Error>> {
        let response = reqwest::get("https://poe.ninja/api/Data/GetStats").await?;
        let str = response.json::<PoeNinjaGetStats>().await?;
        ChangeId::from_str(&str.next_change_id)
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_fetch_latest_change_id_async() {
        let latest_change_id = super::PoeNinjaClient::fetch_latest_change_id_async().await;
        assert!(latest_change_id.is_ok());
        assert!(latest_change_id.unwrap().inner.len() > 50);
    }
}
