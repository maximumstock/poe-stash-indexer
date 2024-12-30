use async_trait::async_trait;
use lapin::{options::BasicPublishOptions, BasicProperties, Channel, Connection};
use stash_api::common::stash::Stash;

use crate::config::{ensure_string_from_env, read_string_from_env};

use super::sink::Sink;

const EXCHANGE: &str = "amq.fanout";

pub struct RabbitMqSink {
    #[allow(dead_code)]
    connection: Connection,
    channel: Channel,
    config: RabbitMqConfig,
}

impl RabbitMqSink {
    #[tracing::instrument]
    pub async fn connect(config: RabbitMqConfig) -> Result<Self, lapin::Error> {
        let connection = lapin::Connection::connect(
            &config.connection_url,
            lapin::ConnectionProperties::default(),
        )
        .await?;

        let channel = connection.create_channel().await?;

        Ok(Self {
            connection,
            channel,
            config,
        })
    }
}

#[async_trait]
impl Sink for RabbitMqSink {
    #[tracing::instrument(skip(self, payload), name = "sink-handle-rabbitmq")]
    async fn handle(&mut self, payload: &[Stash]) -> Result<usize, Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(payload)?;

        self.channel
            .basic_publish(
                EXCHANGE,
                &self.config.producer_routing_key,
                BasicPublishOptions::default(),
                serialized.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .map(|_| payload.len())
            .map_err(|e| e.into())
    }

    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RabbitMqConfig {
    pub connection_url: String,
    pub producer_routing_key: String,
}

impl RabbitMqConfig {
    pub fn from_env() -> Result<Option<RabbitMqConfig>, std::env::VarError> {
        if let Ok(string) = std::env::var("RABBITMQ_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let connection_url = ensure_string_from_env("RABBITMQ_URL");
            let producer_routing_key = read_string_from_env("RABBITMQ_PRODUCER_ROUTING_KEY")
                .unwrap_or("poe-stash-indexer".into());

            Ok(Some(RabbitMqConfig {
                connection_url,
                producer_routing_key,
            }))
        } else {
            Ok(None)
        }
    }
}
