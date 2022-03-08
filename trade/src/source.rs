use std::{fs::File, io::BufReader, path::Path};

use lapin::{
    options::{BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties, Consumer, Result,
};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize)]
pub struct StashRecord {
    pub stash_id: String,
    pub league: String,
    pub account_name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub type_line: String,
    pub note: Option<String>,
    pub stack_size: Option<u32>,
}

pub struct ExampleStream {
    stash_records: Vec<StashRecord>,
}

impl IntoIterator for ExampleStream {
    type Item = StashRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.stash_records.into_iter()
    }
}

impl ExampleStream {
    pub fn new<T: AsRef<Path>>(file_path: T) -> Self {
        let reader = BufReader::new(File::open(file_path).unwrap());
        let stash_records = serde_json::de::from_reader::<_, Vec<StashRecord>>(reader).unwrap();

        Self { stash_records }
    }
}

pub async fn retry_setup_consumer() -> Result<Consumer> {
    let mut consumer = setup_consumer().await;

    while let Err(e) = consumer {
        error!("Encountered an error when connecting to RabbitMQ: {:?}", e);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        consumer = setup_consumer().await;
    }

    consumer
}

pub async fn setup_consumer() -> Result<Consumer> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://poe:poe@rabbitmq".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?; // Note the `with_tokio()` here
    let channel = conn.create_channel().await?;
    let queue_name = "trade_queue";

    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("Declared {:?}", queue_name);

    channel
        .queue_bind(
            queue_name,
            "amq.fanout",
            "stash-record-stream",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Connected to {:?}", queue_name);

    let consumer = channel
        .basic_consume(
            queue_name,
            "trade_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(consumer)
}
