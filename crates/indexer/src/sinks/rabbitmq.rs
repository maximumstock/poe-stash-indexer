use amiquip::{Channel, Connection, Publish};
use async_trait::async_trait;

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
    pub fn connect(config: RabbitMqConfig) -> Result<Self, amiquip::Error> {
        let mut connection = Connection::insecure_open(config.connection_url.as_str())?;
        let channel = connection.open_channel(None)?;

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
                Publish::new(
                    serialized.as_bytes(),
                    self.config.producer_routing_key.as_str(),
                ),
            )
            .map(|_| payload.len())
            .map_err(|e| e.into())
    }
}
