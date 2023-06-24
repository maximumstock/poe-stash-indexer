use std::{sync::Arc, time::Duration};

use bytes::BytesMut;
use reqwest::StatusCode;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tracing::{debug, error, error_span, info, trace_span};

use crate::common::parse::parse_change_id_from_bytes;
use crate::common::poe_api::{get_oauth_token, user_agent, OAuthResponse};
use crate::common::{ChangeId, StashTabResponse};

#[derive(Debug)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub credentials: Option<OAuthResponse>,
}

impl Config {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            credentials: None,
        }
    }
}

#[derive(Default, Debug)]
pub struct Indexer;

#[cfg(feature = "async")]
impl Indexer {
    pub fn new() -> Self {
        Self {}
    }

    /// Start the indexer with a given change_id
    pub async fn start_at_change_id(
        &self,
        mut config: Config,
        change_id: ChangeId,
    ) -> Receiver<IndexerMessage> {
        // Workaround to not have to use [tracing::instrument]
        trace_span!("start_at_change_id", change_id = change_id.inner.as_str());

        info!("Starting at change id: {}", change_id);

        let credentials = get_oauth_token(&config.client_id, &config.client_secret)
            .await
            .expect("Fetch OAuth credentials");
        config.credentials = Some(credentials);

        let (tx, rx) = channel(42);

        schedule_job(tx, change_id, Arc::new(RwLock::new(config)));
        rx
    }
}

fn schedule_job(tx: Sender<IndexerMessage>, next_change_id: ChangeId, config: Arc<RwLock<Config>>) {
    tokio::spawn(async move { process(next_change_id, tx, config).await });
}

#[tracing::instrument(skip(tx))]
async fn process(
    change_id: ChangeId,
    tx: Sender<IndexerMessage>,
    config: Arc<RwLock<Config>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // check if stopping
    if tx.is_closed() {
        return Ok(());
    }

    let url = format!(
        "https://api.pathofexile.com/public-stash-tabs?id={}",
        &change_id
    );
    debug!("Requesting {}", url);

    // TODO: static client somewhere
    let client = reqwest::ClientBuilder::new().build()?;
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header("User-Agent", user_agent(&config.read().await.client_id))
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                config
                    .read()
                    .await
                    .credentials
                    .as_ref()
                    .expect("OAuth credentials are set")
                    .access_token
            )
            .as_str(),
        )
        .send()
        .await;

    let mut response = match response {
        Err(e) => {
            error!("Error when fetching change_id {}: {:?}", change_id, e);
            error_span!("handle_fetch_error").in_scope(|| {
                error!("Error response: {:?}", e);
                error!(fetch_error = ?e);
                schedule_job(tx, change_id, config);
            });
            return Ok(());
        }
        Ok(data) => data,
    };

    if response.status() != StatusCode::OK {
        info!(
            "Rescheduling in 60s due to HTTP response status code {}",
            response.status().as_u16()
        );
        tokio::time::sleep(Duration::from_secs(60)).await;
        schedule_job(tx, change_id, config);
        return Ok(());
    }

    let mut bytes = BytesMut::new();
    let mut prefetch_done = false;
    while let Some(chunk) = response.chunk().await? {
        bytes.extend_from_slice(&chunk);

        if bytes.len() > 120 && !prefetch_done {
            let next_change_id = parse_change_id_from_bytes(&bytes).unwrap();
            tracing::trace!(next_change_id = ?next_change_id);
            prefetch_done = true;
            schedule_job(tx.clone(), next_change_id, config.clone());
        }
    }

    let deserialised = match serde_json::from_slice::<StashTabResponse>(&bytes) {
        Ok(deserialised) => deserialised,
        Err(e) => {
            info!(
                "Rescheduling in 5s due to deserialization issue {}",
                e.to_string()
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
            schedule_job(tx, change_id, config);
            return Ok(());
        }
    };
    debug!(
        "Read response {} with {} stashes",
        deserialised.next_change_id,
        deserialised.stashes.len()
    );
    tracing::trace!(number_stashes = ?deserialised.stashes.len());

    // reschedule if payload is empty
    if deserialised.stashes.is_empty() {
        info!("Empty response, rescheduling {change_id} in 2s");
        tokio::time::sleep(Duration::from_secs(2)).await;
        schedule_job(tx, change_id, config);
        return Ok(());
    }

    tx.try_send(IndexerMessage::Tick {
        response: deserialised,
        previous_change_id: change_id.clone(),
        change_id,
        created_at: std::time::SystemTime::now(),
    })
    .expect("Sending IndexerMessage failed");

    Ok(())
}

#[derive(Debug, Clone)]
pub enum IndexerMessage {
    Tick {
        response: StashTabResponse,
        change_id: ChangeId,
        previous_change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    RateLimited(Duration),
    Stop,
}
