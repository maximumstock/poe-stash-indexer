use async_trait::async_trait;

use crate::common::ChangeId;

/// An async HTTP client for Path of Exile's Public Stash Tab API

#[async_trait]
trait PSTClient {
    async fn get_change_id(&self, change_id: &ChangeId) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct AsyncClient {
    pub(crate) config: AsyncClientConfig,
    pub(crate) client: reqwest::Client,
    pub(crate) access_token: Option<String>,
}

impl AsyncClient {
    pub fn new(config: AsyncClientConfig) -> Self {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        Self {
            config,
            client,
            access_token: None,
        }
    }

    pub async fn authorize(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[async_trait]
impl PSTClient for AsyncClient {
    async fn get_change_id(&self, change_id: &ChangeId) -> Result<(), Box<dyn std::error::Error>> {
        self.authorize().await?;

        let url = format!("{}/public-stash-tabs?id={}", self.config.url, change_id);
        let res = self
            .client
            .get(&url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .send()
            .await?;

        if res.

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AsyncClientConfig {
    url: String,
}
