use crate::change_id::ChangeId;
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
        let response = ureq::get("https://poe.ninja/api/Data/GetStats")
            .call()
            .expect("Failed to fetch latest change id from poe.ninja");
        let stats: PoeNinjaGetStats = serde_json::from_reader(response.into_reader())?;
        ChangeId::from_str(&stats.next_change_id)
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
