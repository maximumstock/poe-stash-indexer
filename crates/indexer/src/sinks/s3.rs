use std::{
    collections::HashMap,
    fmt::Debug,
    io::Write,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client};
use aws_types::region::Region;
use chrono::NaiveDateTime;
use flate2::Compression;
use futures::{stream::FuturesUnordered, StreamExt};
use stash_api::common::stash::Stash;
use tracing::{error, info};

use crate::config::ensure_string_from_env;

use super::sink::Sink;

pub struct S3Sink {
    client: Client,
    bucket: String,
    buffer: Arc<RwLock<HashMap<String, Vec<Stash>>>>,
    last_sync: Option<NaiveDateTime>,
}

impl S3Sink {
    #[tracing::instrument]
    pub async fn connect(
        bucket: impl Into<String> + Debug,
        region: impl Into<String> + Debug,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bucket = bucket.into();

        let mut config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        config = config
            .into_builder()
            .region(Some(Region::new(region.into())))
            .build();
        let client = aws_sdk_s3::Client::new(&config);

        Ok(Self {
            client,
            bucket,
            buffer: Default::default(),
            last_sync: None,
        })
    }

    async fn sync(&mut self) {
        // todo: error handling of s3 client
        // todo: sync in another tokio task that does not block the rest
        info!("Syncing S3 Sink");
        let tasks = self
            .buffer
            .read()
            .unwrap()
            .iter()
            .filter(|(_, stashes)| !stashes.is_empty())
            .map(|(league, stashes)| {
                let key = format!(
                    "{}/{}.json.gz",
                    league,
                    stashes.last().unwrap().created_at.format(TIME_BUCKET),
                );
                let mut w = Vec::new();
                stashes.iter().for_each(|s| {
                    let _ = jsonl::write(&mut w, s);
                });
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(&w).unwrap();
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
                    "Error when flushing S3 sink with league {}: {:?} - will re-attempt sync next interval",
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
    async fn handle(&mut self, payload: &[Stash]) -> Result<usize, Box<dyn std::error::Error>> {
        if payload.is_empty() {
            return Ok(0);
        }

        let should_sync = match &self.last_sync {
            Some(last_sync) => {
                let batch_timestamp = payload.first().unwrap().created_at;
                let sync_needed = format!("{}", batch_timestamp.format(TIME_BUCKET))
                    > format!("{}", last_sync.format(TIME_BUCKET));
                if sync_needed {
                    self.last_sync = Some(batch_timestamp);
                }
                sync_needed
            }
            None => {
                self.last_sync = Some(chrono::offset::Utc::now().naive_utc());
                false
            }
        };

        if should_sync {
            self.sync().await;
        }

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

        Ok(payload.len())
    }

    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sync().await;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket_name: String,
    pub region: String,
}

impl S3Config {
    pub fn from_env() -> Result<Option<S3Config>, std::env::VarError> {
        if let Ok(string) = std::env::var("S3_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let bucket_name = ensure_string_from_env("S3_SINK_BUCKET_NAME");
            let region = ensure_string_from_env("S3_SINK_REGION");

            Ok(Some(S3Config {
                bucket_name,
                region,
            }))
        } else {
            Ok(None)
        }
    }
}
