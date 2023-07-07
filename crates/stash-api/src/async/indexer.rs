use std::{sync::Arc, time::Duration};

use bytes::BytesMut;
use reqwest::StatusCode;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tracing::{debug, error, error_span, info, trace_span};
use trade_common::telemetry::generate_http_client;

use crate::common::parse::parse_change_id_from_bytes;
use crate::common::poe_api::{get_oauth_token, user_agent, OAuthResponse};
use crate::common::{ChangeId, StashTabResponse};

#[derive(Default, Debug)]
pub struct Indexer {
    pub(crate) is_stopping: bool,
}

pub type IndexerResult = Receiver<IndexerMessage>;

#[cfg(feature = "async")]
impl Indexer {
    pub fn new() -> Self {
        Self {}
    }

    /// Start the indexer with a given change_id
    pub async fn start_at_change_id(
        &self,
        client_id: String,
        client_secret: String,
        change_id: ChangeId,
    ) -> Receiver<IndexerMessage> {
        // Workaround to not have to use [tracing::instrument]
        trace_span!("start_at_change_id", change_id = change_id.inner.as_str());

        info!("Starting at change id: {}", change_id);

        let credentials = get_oauth_token(&client_id, &client_secret)
            .await
            .expect("Fetch OAuth credentials");
        info!("Fetched OAuth credentials");
        let credentials = Some(credentials);

        let (tx, rx) = channel(42);

        schedule_job(
            jobs,
            tx,
            change_id,
            client_id,
            client_secret,
            Arc::new(RwLock::new(credentials)),
        );
        rx
    }
}

fn schedule_job(
    jobs: Arc<Mutex<VecDeque<ChangeId>>>,
    tx: Sender<IndexerMessage>,
    next_change_id: ChangeId,
    client_id: String,
    client_secret: String,
    config: Arc<RwLock<Option<OAuthResponse>>>,
) {
    tokio::spawn(async move {
        jobs.lock().await.push_back(next_change_id);
        process(jobs, tx, client_id, client_secret, config).await
    });
}

#[tracing::instrument(skip(tx))]
async fn process(
    jobs: Arc<Mutex<VecDeque<ChangeId>>>,
    mut tx: Sender<IndexerMessage>,
    client_id: String,
    client_secret: String,
    config: Arc<RwLock<Option<OAuthResponse>>>,
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

    if let Some(change_id) = next {
        let url = format!(
            "https://api.pathofexile.com/public-stash-tabs?id={}",
            &change_id
        );
        tracing::trace!(change_id = ?change_id, url = ?url);
        debug!("Requesting {}", url);

        // TODO: static client somewhere
        let client = reqwest::ClientBuilder::new().build()?;
        let response = client
            .get(url)
            .header("Accept", "application/json")
            .header("User-Agent", user_agent(&client_id))
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    config
                        .read()
                        .await
                        .as_ref()
                        .expect("OAuth credentials are set")
                        .access_token
                )
                .as_str(),
            )
            .as_str(),
        )
        .send()
        .await;

        let mut response = match response {
            Err(e) => {
                error_span!("handle_fetch_error").in_scope(|| {
                    error!("Error response: {:?}", e);
                    tracing::trace!(fetch_error = ?e);
                    schedule_job(jobs, tx, change_id, client_id, client_secret, config);
                });
                return Ok(());
            }
            Ok(data) => data,
        };

        let mut bytes = BytesMut::new();
        let mut prefetch_done = false;
        while let Some(chunk) = response.chunk().await? {
            bytes.extend_from_slice(&chunk);

            if bytes.len() > 120 && !prefetch_done {
                let next_change_id = parse_change_id_from_bytes(&bytes).unwrap();
                tracing::trace!(next_change_id = ?next_change_id);
                prefetch_done = true;
                schedule_job(
                    jobs.clone(),
                    tx.clone(),
                    next_change_id,
                    client_id.clone(),
                    client_secret.clone(),
                    config.clone(),
                );
            }
        }

        // TODO: handle errors by rescheduling based on error
        let response = serde_json::from_slice::<StashTabResponse>(&bytes)?;
        debug!(
            "Read response {} with {} stashes",
            response.next_change_id,
            response.stashes.len()
        );
        tracing::trace!(number_stashes = ?response.stashes.len());

        // reschedule if payload is empty
        if response.stashes.is_empty() {
            info!("Empty response, rescheduling {change_id}");
            tokio::time::sleep(Duration::from_secs(2)).await;
            schedule_job(jobs, tx, change_id, client_id, client_secret, config);
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
