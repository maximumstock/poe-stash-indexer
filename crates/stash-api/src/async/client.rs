use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::common::{
    pst_api::{user_agent, OAuthRequestPayload, OAuthResponse},
    ChangeId, StashTabResponse,
};

/// An async HTTP client for Path of Exile's Public Stash Tab API

pub enum PSTError {
    RateLimited,
    InternalServerError,
    BadGateway,
    UnexpectedError,
    BadRequest,
}

pub type Result<T> = std::result::Result<T, PSTError>;

#[async_trait]
trait PSTClient {
    async fn get_change_id(&self, change_id: &ChangeId) -> Result<StashTabResponse>;
}

#[derive(Debug, Clone)]
pub struct AsyncClient {
    pub(crate) config: AsyncClientConfig,
    pub(crate) client: reqwest::Client,
    pub(crate) access_token: Arc<RwLock<Option<String>>>,
    pub(crate) user_agent: String,
}

impl AsyncClient {
    pub fn new(config: AsyncClientConfig) -> Self {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        Self {
            user_agent: user_agent(&config.client_id),
            config,
            client,
            access_token: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn authorize(&self) -> Result<OAuthResponse> {
        let url = "https://www.pathofexile.com/oauth/token";

        let payload = serde_urlencoded::to_string(OAuthRequestPayload::new(
            self.config.client_id.clone(),
            self.config.client_secret.clone(),
        ))
        .unwrap();

        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", user_agent(&self.config.client_id))
            .body(payload)
            .send()
            .await
            .map_err(|_| PSTError::UnexpectedError)?;

        let oauth_response = match res.status() {
            StatusCode::OK => Ok(res
                .json::<OAuthResponse>()
                .await
                .map_err(|_| PSTError::UnexpectedError)?),
            StatusCode::TOO_MANY_REQUESTS => Err(PSTError::RateLimited),
            StatusCode::INTERNAL_SERVER_ERROR => Err(PSTError::InternalServerError),
            StatusCode::BAD_GATEWAY => Err(PSTError::BadGateway),
            _ => Err(PSTError::UnexpectedError),
        }?;

        self.access_token
            .write()
            .await
            .replace(oauth_response.access_token.clone());

        Ok(oauth_response)
    }
}

#[async_trait]
impl PSTClient for AsyncClient {
    async fn get_change_id(&self, change_id: &ChangeId) -> Result<StashTabResponse> {
        if self.access_token.read().await.is_none() {
            self.authorize().await?;
        }

        let url = format!("{}/public-stash-tabs?id={}", self.config.url, change_id);
        let access_token = self.access_token.read().await;
        let res = self
            .client
            .get(&url)
            .bearer_auth(access_token.as_ref().unwrap())
            .send()
            .await
            .map_err(|_| PSTError::UnexpectedError)?;

        match res.status() {
            StatusCode::OK => Ok(res
                .json::<StashTabResponse>()
                .await
                .map_err(|_| PSTError::UnexpectedError)?),
            StatusCode::TOO_MANY_REQUESTS => Err(PSTError::RateLimited),
            StatusCode::INTERNAL_SERVER_ERROR => Err(PSTError::InternalServerError),
            StatusCode::BAD_GATEWAY => Err(PSTError::BadGateway),
            _ => Err(PSTError::UnexpectedError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncClientConfig {
    url: String,
    client_id: String,
    client_secret: String,
}
