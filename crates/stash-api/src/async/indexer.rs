use std::str::FromStr;
use std::{sync::Arc, time::Duration};

use chrono::Utc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tracing::{debug, error, error_span, info, trace, trace_span};
use trade_common::secret::SecretString;
use trade_common::telemetry::generate_http_client;
use trade_common::{ClientWithMiddleware, RateLimiter};

use crate::common::parse::parse_change_id_from_bytes;
use crate::common::stash::Stash;
use crate::common::ChangeId;
use crate::poe_api::auth::{get_oauth_token, user_agent, OAuthResponse};
use crate::poe_api::poe_stash_api::protocol::PublicStashTabResponse;

#[derive(Debug)]
pub struct Indexer {
    pub(crate) client_id: String,
    pub(crate) client_secret: SecretString,
    pub(crate) developer_mail: SecretString,
}

impl Indexer {
    pub fn new(
        client_id: String,
        client_secret: SecretString,
        developer_mail: SecretString,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            developer_mail,
        }
    }

    /// Start the indexer with a given change_id
    pub async fn start_at_change_id(&self, change_id: ChangeId) -> Receiver<IndexerMessage> {
        // Workaround to not have to use [tracing::instrument]
        trace_span!("start_at_change_id", change_id = change_id.inner.as_str());

        info!("Starting at change id: {}", change_id);

        let credentials =
            get_oauth_token(&self.client_id, &self.client_secret, &self.developer_mail)
                .await
                .expect("Fetch OAuth credentials");

        let rate_limiter = RateLimiter::builder()
            .initial(0)
            .max(1)
            .refill(1)
            .interval(Duration::from_secs(1))
            .build();

        let client = Arc::new(generate_http_client(Some(rate_limiter)));

        let (tx, rx) = channel(100);

        schedule_job(
            tx,
            change_id,
            self.client_id.clone(),
            self.client_secret.clone(),
            self.developer_mail.clone(),
            Arc::new(RwLock::new(Some(credentials))),
            client,
        );
        rx
    }
}

fn schedule_job(
    tx: Sender<IndexerMessage>,
    next_change_id: ChangeId,
    client_id: String,
    client_secret: SecretString,
    developer_mail: SecretString,
    config: Arc<RwLock<Option<OAuthResponse>>>,
    client: Arc<ClientWithMiddleware>,
) {
    tokio::spawn(process(
        next_change_id,
        tx,
        client_id,
        client_secret,
        developer_mail,
        config,
        client,
    ));
}

#[tracing::instrument(skip(tx, config))]
async fn process(
    change_id: ChangeId,
    tx: Sender<IndexerMessage>,
    client_id: String,
    client_secret: SecretString,
    developer_mail: SecretString,
    config: Arc<RwLock<Option<OAuthResponse>>>,
    client: Arc<ClientWithMiddleware>,
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

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            user_agent(&client_id, developer_mail.expose()),
        )
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
        .send()
        .await;

    let mut response = match response {
        Err(e) => {
            error!("Error when fetching change_id {}: {:?}", change_id, e);
            error_span!("handle_fetch_error").in_scope(|| {
                error!("Error response: {:?}", e);
                error!(fetch_error = ?e);
                schedule_job(
                    tx,
                    change_id,
                    client_id,
                    client_secret,
                    developer_mail,
                    config,
                    client,
                );
            });
            return Ok(());
        }
        Ok(data) => data,
    };

    if !response.status().is_success() {
        info!(
            "Rescheduling in 60s due to HTTP response status code {}",
            response.status().as_u16()
        );
        tokio::time::sleep(Duration::from_secs(60)).await;
        schedule_job(
            tx,
            change_id,
            client_id,
            client_secret,
            developer_mail,
            config,
            client,
        );
        return Ok(());
    }

    let mut bytes = vec![];
    let mut prefetch_done = false;
    while let Some(chunk) = response.chunk().await? {
        bytes.extend(chunk);

        if bytes.len() > 60 && !prefetch_done {
            if seems_empty(&bytes) {
                debug!("Rescheduling in 4s due to empty response");
                tokio::time::sleep(Duration::from_secs(4)).await;
                schedule_job(
                    tx,
                    change_id,
                    client_id,
                    client_secret,
                    developer_mail,
                    config,
                    client,
                );
                return Ok(());
            } else {
                let next_change_id = parse_change_id_from_bytes(&bytes).unwrap();
                tracing::trace!(next_change_id = ?next_change_id);
                prefetch_done = true;
                schedule_job(
                    tx.clone(),
                    next_change_id,
                    client_id.clone(),
                    client_secret.clone(),
                    developer_mail.clone(),
                    config.clone(),
                    client.clone(),
                );
            }
        }
    }

    let deserialised = match serde_json::from_slice::<PublicStashTabResponse>(&bytes) {
        Ok(deserialised) => deserialised,
        Err(e) => {
            info!("Rescheduling in 5s due to deserialization issue {:?}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            schedule_job(
                tx,
                change_id,
                client_id,
                client_secret,
                developer_mail,
                config,
                client,
            );
            return Ok(());
        }
    };
    debug!(
        "Read response {} with {} stashes",
        deserialised.next_change_id,
        deserialised.stashes.len()
    );
    trace!(number_stashes = ?deserialised.stashes.len());

    let next_change_id =
        ChangeId::from_str(&deserialised.next_change_id).expect("Invalid next_change_id");
    let now = Utc::now().naive_utc();
    let stashes = deserialised
        .stashes
        .into_iter()
        .map(|s| Stash {
            account_name: s.account_name,
            id: s.id,
            stash: s.stash,
            stash_type: s.stash_type,
            items: s.items,
            public: s.public,
            league: s.league,
            created_at: now,
            change_id: change_id.to_string(),
            next_change_id: deserialised.next_change_id.clone(),
        })
        .collect::<Vec<_>>();

    tx.try_send(IndexerMessage::Tick {
        stashes,
        change_id,
        next_change_id,
        created_at: std::time::SystemTime::now(),
    })
    .expect("Sending IndexerMessage failed");

    Ok(())
}

fn seems_empty(bytes: &[u8]) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.contains("stashes:[]") || text.contains("\"stashes\":[]"),
        Err(_) => false,
    }
}

#[derive(Debug, Clone)]
pub enum IndexerMessage {
    Tick {
        stashes: Vec<Stash>,
        /// The [`ChangeId`] of the current set of stashes
        change_id: ChangeId,
        /// The [`ChangeId`] of the next set of stashes after this tick
        next_change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    RateLimited(Duration),
    Stop,
}
