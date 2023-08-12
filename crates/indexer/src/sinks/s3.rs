use std::{
    collections::HashMap,
    fmt::Debug,
    io::Write,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use aws_sdk_s3::{primitives::ByteStream, Client};
use chrono::NaiveDateTime;
use flate2::Compression;
use futures::{stream::FuturesUnordered, StreamExt};
use tracing::{error, info};
use trade_common::secret::SecretString;

use crate::stash_record::StashRecord;

use super::sink::Sink;

pub struct S3Sink {
    client: Client,
    bucket: String,
    buffer: Arc<RwLock<HashMap<String, Vec<StashRecord>>>>,
    last_sync: Option<NaiveDateTime>,
}

impl S3Sink {
    #[tracing::instrument]
    pub async fn connect(
        bucket: impl Into<String> + Debug,
        access_key: impl Into<String> + Debug,
        secret_key: SecretString,
    ) -> Result<Self, lapin::Error> {
        let bucket = bucket.into();
        let access_key = access_key.into();
        let secret_key = secret_key;

        let credentials = aws_credential_types::Credentials::new(
            &access_key,
            secret_key.expose(),
            None,
            None,
            "poe-stash-indexer",
        );
        let credentials_provider =
            aws_credential_types::provider::SharedCredentialsProvider::new(credentials);
        let config = aws_config::from_env()
            .credentials_provider(credentials_provider)
            .load()
            .await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket,
            buffer: Default::default(),
            last_sync: None,
        })
    }

    async fn sync(&mut self) {
        info!("Syncing S3 Sink");
        // todo: error handling of s3 client
        let tasks = self
            .buffer
            .read()
            .unwrap()
            .iter()
            .filter(|(_, stashes)| !stashes.is_empty())
            .map(|(league, stashes)| {
                let key = format!(
                    "{}/{}.json.gzip",
                    league,
                    stashes.last().unwrap().created_at.format(TIME_BUCKET),
                );
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
                encoder
                    .write_all(&serde_json::to_vec(&stashes).unwrap())
                    .unwrap();
                let compressed = encoder.finish().unwrap();
                let payload = ByteStream::from(compressed);
                (league.clone(), key, payload)
            })
            .collect::<Vec<_>>();

        let mut tasks = tasks
            .into_iter()
            .map(|(league, key, payload)| {
                let f = self
                    .client
                    .put_object()
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
            } else if let Some(entry) = self.buffer.write().unwrap().get_mut(&league) {
                entry.clear();
            }
        }
    }
}

const TIME_BUCKET: &str = "%Y/%m/%d/%H/%M";

#[async_trait]
impl Sink for S3Sink {
    #[tracing::instrument(skip(self, payload), name = "sink-handle-s3")]
    async fn handle(
        &mut self,
        payload: &[StashRecord],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if payload.is_empty() {
            return Ok(0);
        }

        let should_sync = match &self.last_sync {
            Some(last_sync) => {
                let batch_timestamp = payload.first().unwrap().created_at;
                let next = format!("{}", batch_timestamp.format(TIME_BUCKET))
                    > format!("{}", last_sync.format(TIME_BUCKET));
                if next {
                    self.last_sync = Some(batch_timestamp);
                }
                next
            }
            None => {
                self.last_sync = Some(chrono::offset::Utc::now().naive_utc());
                true
            }
        };

        if should_sync {
            self.sync().await;
        } else {
            for stash in payload {
                if let Some(league) = &stash.league {
                    self.buffer
                        .write()
                        .unwrap()
                        .entry(league.clone())
                        .or_default()
                        // todo: box stash records
                        .push(stash.clone());
                }
            }
        }

        Ok(payload.len())
    }

    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sync().await;
        Ok(())
    }
}
