use amiquip::{Channel, Connection, Publish};

use crate::config::RabbitMqConfig;

use super::sink::Sink;

const EXCHANGE: &str = "amq.fanout";

pub struct RabbitMq<'a> {
    #[allow(dead_code)]
    connection: Connection,
    channel: Channel,
    config: &'a RabbitMqConfig,
}

impl<'a> RabbitMq<'a> {
    #[tracing::instrument]
    pub fn connect(config: &'a RabbitMqConfig) -> Result<Self, amiquip::Error> {
        let mut connection = Connection::insecure_open(config.connection_url.as_str())?;
        let channel = connection.open_channel(None)?;

        Ok(Self {
            connection,
            channel,
            config,
        })
    }
}

impl<'a> Sink for RabbitMq<'a> {
    #[tracing::instrument(skip(self, payload), name = "handle-rabbitmq")]
    fn handle(
        &self,
        payload: &[crate::stash_record::StashRecord],
    ) -> Result<usize, Box<dyn std::error::Error>> {
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
