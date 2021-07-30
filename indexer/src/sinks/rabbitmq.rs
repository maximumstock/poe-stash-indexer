use amiquip::{Channel, Connection, Exchange, Publish};

use super::sink::Sink;

pub struct RabbitMq {
    #[allow(dead_code)]
    connection: Connection,
    channel: Channel,
}

impl RabbitMq {
    pub fn connect(connection_url: &str) -> Result<Self, amiquip::Error> {
        let mut connection = Connection::insecure_open(connection_url)?;
        let channel = connection.open_channel(None)?;

        Ok(Self {
            connection,
            channel,
        })
    }
}

impl Sink for RabbitMq {
    fn handle(
        &self,
        payload: &[crate::stash_record::StashRecord],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(payload)?;
        let exchange = Exchange::direct(&self.channel);

        exchange
            .publish(Publish::new(serialized.as_bytes(), "myroute"))
            .map(|_| payload.len())
            .map_err(|e| e.into())
    }
}
