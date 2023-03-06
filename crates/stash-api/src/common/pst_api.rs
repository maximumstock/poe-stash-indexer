use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthRequestPayload {
    client_id: String,
    client_secret: String,
    grant_type: String,
    scope: String,
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

pub fn user_agent(client_id: &str) -> String {
    format!("OAuth {client_id}/0.1 (contact: mxmlnstock@gmail.com)")
}
