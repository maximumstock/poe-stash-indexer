use async_trait::async_trait;
use lapin::{options::BasicPublishOptions, BasicProperties, Channel, Connection};

use crate::{config::RabbitMqConfig, stash_record::StashRecord};

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
    #[tracing::instrument(skip(self, payload), name = "handle-rabbitmq")]
    async fn handle(&self, payload: &[StashRecord]) -> Result<usize, Box<dyn std::error::Error>> {
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
}
