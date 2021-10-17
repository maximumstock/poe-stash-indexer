use amiquip::{Channel, Connection, Publish};

use crate::config::RabbitMqConfig;

use super::sink::Sink;

const EXCHANGE: &str = "amq.fanout";

pub struct RabbitMq {
    #[allow(dead_code)]
    connection: Connection,
    channel: Channel,
    config: RabbitMqConfig,
}

impl RabbitMq {
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

impl Sink for RabbitMq {
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
