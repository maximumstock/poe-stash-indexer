use std::time::Duration;

use serde::{Deserialize, Serialize};
use trade_common::secret::SecretString;

pub fn user_agent(client_id: &str) -> String {
    format!("OAuth {client_id}/0.1 (contact: mxmlnstock@gmail.com)")
}

#[derive(Debug, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthRequestPayload {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub scope: String,
}

impl OAuthRequestPayload {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            grant_type: "client_credentials".into(),
            scope: "service:psapi".into(),
        }
    }
}

/// According to https://www.pathofexile.com/developer/docs/authorization
pub async fn get_oauth_token(
    client_id: &str,
    client_secret: &SecretString,
) -> Result<OAuthResponse, Box<dyn std::error::Error>> {
    use trade_common::telemetry::generate_http_client;

    let url = "https://www.pathofexile.com/oauth/token";
    let payload = serde_urlencoded::to_string(OAuthRequestPayload::new(
        client_id.into(),
        client_secret.expose().to_string(),
    ))
    .unwrap();

    let response = generate_http_client()
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", user_agent(client_id).as_str())
        .body(payload)
        .send()
        .await?;

    serde_json::from_slice(&response.bytes().await?).map_err(|e| e.into())
}

const DEFAULT_RATE_LIMIT_TIMER: u64 = 60;

pub fn parse_rate_limit_timer(input: Option<&str>) -> Duration {
    let seconds = input
        .and_then(|v| v.split(':').last())
        .map(|s| {
            if s.ne("60") {
                tracing::warn!("Expected x-rate-limit-ip to be 60 seconds");
            }
            s.parse().unwrap_or(DEFAULT_RATE_LIMIT_TIMER)
        })
        .unwrap_or(DEFAULT_RATE_LIMIT_TIMER);

    Duration::from_secs(seconds)
}
