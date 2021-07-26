use crate::common::ChangeId;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct PoeNinjaGetStats {
    next_change_id: String,
}

#[derive(Debug)]
pub(crate) struct PoeNinjaClient {}

impl PoeNinjaClient {
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
}

#[cfg(test)]
mod test {
    #[test]
    fn test_fetch_latest_change_id() {
        let latest_change_id = super::PoeNinjaClient::fetch_latest_change_id();
        assert!(latest_change_id.is_ok());
        assert!(latest_change_id.unwrap().inner.len() > 50);
    }
}
