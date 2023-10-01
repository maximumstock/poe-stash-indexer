use std::{
    collections::HashMap,
    fmt::Debug,
    io::Write,
    sync::{Arc, RwLock},
};

use crate::differ::DiffEvent;
use aws_sdk_s3::{primitives::ByteStream, Client};
use aws_types::region::Region;
use chrono::NaiveDateTime;
use flate2::Compression;
use futures::{stream::FuturesUnordered, StreamExt};
use tracing::{error, info};
use trade_common::secret::SecretString;

const TIME_BUCKET: &str = "%Y/%m/%d/%H/%M";

pub struct S3Sink {
    client: Client,
    bucket: String,
    buffer: Arc<RwLock<HashMap<String, Vec<DiffEvent>>>>,
    last_sync: Option<NaiveDateTime>,
}

impl S3Sink {
    #[tracing::instrument]
    pub async fn connect(
        bucket: impl Into<String> + Debug,
        access_key: impl Into<String> + Debug,
        secret_key: SecretString,
        region: impl Into<String> + Debug,
    ) -> Self {
        let bucket = bucket.into();
        let access_key = access_key.into();
        let secret_key = secret_key;

        let credentials = aws_credential_types::Credentials::new(
            &access_key,
            secret_key.expose(),
            None,
            None,
            "poe-stash-differ",
        );
        let credentials_provider =
            aws_credential_types::provider::SharedCredentialsProvider::new(credentials);
        let config = aws_config::from_env()
            .region(Region::new(region.into()))
            .credentials_provider(credentials_provider)
            .load()
            .await;
        let client = Client::new(&config);

        Self {
            client,
            bucket,
            buffer: Default::default(),
            last_sync: None,
        }
    }

    async fn sync(&mut self) {
        // todo: error handling of s3 client
        // todo: sync in another tokio task that does not block the rest
        info!("Syncing S3 Sink");
        let mut tasks = self
            .buffer
            .read()
            .unwrap()
            .iter()
            .filter(|(_, events)| !events.is_empty())
            .map(|(league, events)| {
                let key = format!(
                    "{}/{}.json.gz",
                    league,
                    events.last().unwrap().timestamp().format(TIME_BUCKET),
                );
                let mut w = Vec::new();
                events.iter().for_each(|s| {
                    let _ = jsonl::write(&mut w, s);
                });
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(&w).unwrap();
                let compressed = encoder.finish().unwrap();
                let payload = ByteStream::from(compressed);
                (league.clone(), key, payload)
            })
            .map(|(league, key, payload)| {
                let f = self
                    .client
                    .put_object()
                    .storage_class(aws_sdk_s3::types::StorageClass::OnezoneIa)
                    .bucket(&self.bucket)
                    .key(key)
                    .body(payload)
                    .send();

                async { (league, f.await) }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some((league, res)) = tasks.next().await {
            if let Err(e) = res {
                error!(
                    "Error when flushing S3 sink with league {}: {} - will re-attempt sync next interval",
                    league, e
                )
            } else {
                self.buffer.write().unwrap().remove(&league);
            }
        }
    }

    #[tracing::instrument(skip(self, events), name = "sink-handle-s3")]
    pub async fn handle(&mut self, events: Vec<DiffEvent>) -> usize {
        if events.is_empty() {
            return 0;
        }

        let should_sync = match &self.last_sync {
            Some(last_sync) => {
                let batch_timestamp = events.first().unwrap().timestamp();
                let next = format!("{}", batch_timestamp.format(TIME_BUCKET))
                    > format!("{}", last_sync.format(TIME_BUCKET));
                if next {
                    self.last_sync = Some(batch_timestamp);
                }
                next
            }
            None => {
                self.last_sync = Some(chrono::offset::Utc::now().naive_utc());
                false
            }
        };

        if should_sync {
            // todo: rewrite to not block immediately, but spawn a task that syncs in the background
            // that should also handle the case when the sync is in background and flush is called manually,
            // ie. when the program stops and tries to save state.
            self.sync().await;
        }

        let n_events = events.len();
        let mut write = self.buffer.write().unwrap();
        events.into_iter().for_each(|event| {
            write
                .entry(event.league().to_string())
                .or_default()
                // todo: box stash records
                .push(event);
        });

        n_events
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.sync().await;
        Ok(())
    }
}
