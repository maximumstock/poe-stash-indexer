use async_trait::async_trait;
use stash_api::common::stash::Stash;

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
