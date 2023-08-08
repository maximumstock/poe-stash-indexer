use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use aws_sdk_s3::{primitives::ByteStream, Client};
use chrono::NaiveDateTime;
use serde::Serialize;
use trade_common::secret::SecretString;

use crate::stash_record::StashRecord;

use super::sink::Sink;

pub struct S3Sink {
    client: Client,
    bucket: String,
    buffer: HashMap<String, Vec<StashRecord>>,
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
}

#[derive(Debug, Serialize)]
struct S3Payload(Vec<StashRecord>);

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
            let payloads = self
                .buffer
                .iter()
                .filter(|(_, v)| !v.is_empty())
                .map(|(k, v)| {
                    let pay = S3Payload(v.clone());
                    let serialized = serde_json::to_vec(&pay).unwrap();
                    let content = ByteStream::from(serialized);
                    // todo: compression
                    // todo: flush on graceful shutdown
                    // todo: error handling of s3 client
                    let key = format!(
                        "{}/{}.json",
                        k,
                        v.last().unwrap().created_at.format(TIME_BUCKET),
                    );

                    self.client
                        .put_object()
                        .bucket(&self.bucket)
                        .key(key)
                        .body(content)
                        .send()
                })
                .collect::<Vec<_>>();

            futures::future::join_all(payloads).await;
            self.buffer.clear();
        } else {
            for stash in payload {
                if let Some(league) = &stash.league {
                    self.buffer
                        .entry(league.clone())
                        .or_default()
                        .push(stash.clone());
                }
            }
        }

        Ok(payload.len())
    }
}
