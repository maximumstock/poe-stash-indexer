use crate::common::ChangeId;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct PoeNinjaGetStats {
    next_change_id: String,
}

#[derive(Debug)]
pub(crate) struct PoeNinjaClient;

impl PoeNinjaClient {
    #[cfg(feature = "sync")]
    pub fn fetch_latest_change_id() -> Result<ChangeId, Box<dyn std::error::Error>> {
        ureq::get("https://poe.ninja/api/Data/GetStats")
            .call()
            .map_err(|e| e.into())
            .and_then(|res| res.into_string().map_err(|e| e.into()))
            .and_then(|s| {
                serde_json::from_str::<PoeNinjaGetStats>(s.as_str()).map_err(|e| e.into())
            })
            .and_then(|x| ChangeId::from_str(&x.next_change_id))
    }

    #[cfg(feature = "async")]
    #[allow(dead_code)]
    pub async fn fetch_latest_change_id_async() -> Result<ChangeId, Box<dyn std::error::Error>> {
        let response = reqwest::get("https://poe.ninja/api/Data/GetStats").await?;
        let str = response.json::<PoeNinjaGetStats>().await?;
        ChangeId::from_str(&str.next_change_id)
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[cfg(feature = "sync")]
    fn test_fetch_latest_change_id_sync() {
        let latest_change_id = super::PoeNinjaClient::fetch_latest_change_id();
        assert!(latest_change_id.is_ok());
        assert!(latest_change_id.unwrap().inner.len() > 50);
    }

    #[test]
    #[cfg(feature = "async")]
    fn test_fetch_latest_change_id_async() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let latest_change_id =
            runtime.block_on(super::PoeNinjaClient::fetch_latest_change_id_async());
        assert!(latest_change_id.is_ok());
        assert!(latest_change_id.unwrap().inner.len() > 50);
    }
}
