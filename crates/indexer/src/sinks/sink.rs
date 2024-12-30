use async_trait::async_trait;
use stash_api::common::stash::Stash;

#[async_trait]
pub trait Sink {
    /// Handles processing a slice of [`Stash`].
    async fn handle(&mut self, payload: &[Stash]) -> Result<usize, Box<dyn std::error::Error>>;

    /// Sinks can be stateful and so want to be flushed upon graceful shutdown.
    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
