use amiquip::{Channel, Connection, Publish};

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

pub struct RabbitMqConfig {
    connection_url: String,
    producer_routing_key: String,
}

impl RabbitMqConfig {
    pub fn from_env() -> Result<Option<RabbitMqConfig>, std::env::VarError> {
        std::env::var("RABBITMQ_SINK_ENABLED")
            .map(|s| s.to_lowercase().eq(&"true") || s.eq(&"1"))
            .and_then(|enabled| {
                if enabled {
                    let connection_url = std::env::var("RABBITMQ_URL")?;
                    let producer_routing_key = std::env::var("RABBITMQ_PRODUCER_ROUTING_KEY")?;

                    Ok(Some(RabbitMqConfig {
                        connection_url,
                        producer_routing_key,
                    }))
                } else {
                    Ok(None)
                }
            })
    }
}
