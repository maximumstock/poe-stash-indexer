use async_trait::async_trait;
use stash_api::common::stash::Stash;

use crate::{
    config::Configuration,
    sinks::{rabbitmq::RabbitMqSink, s3::S3Sink},
};

#[async_trait]
pub trait Sink {
    /// Handles processing a slice of [`Stash`].
    /// Each Sink implementation should handle errors internally, as the caller won't be able to handle it so we don't want
    /// to block on Sink-specific errors.
    /// TODO: make sure the usage of multiple sinks doesn't block each other.
    async fn handle(&mut self, payload: &[Stash]) -> Result<usize, Box<dyn std::error::Error>>;

    /// Sinks can be stateful and so want to be flushed upon graceful shutdown.
    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

pub async fn setup_sinks(
    config: Configuration,
) -> Result<Vec<Box<dyn Sink>>, Box<dyn std::error::Error>> {
    let mut sinks: Vec<Box<dyn Sink>> = vec![];

    if let Some(conf) = config.rabbitmq {
        let mq_sink = RabbitMqSink::connect(conf).await?;
        sinks.push(Box::new(mq_sink));
        tracing::info!("Configured RabbitMQ fanout sink");
    }

    if let Some(config) = config.s3 {
        let s3_sink = S3Sink::connect(&config.bucket_name, &config.region).await?;
        sinks.push(Box::new(s3_sink));
        tracing::info!("Configured S3 sink");
    }

    Ok(sinks)
}
