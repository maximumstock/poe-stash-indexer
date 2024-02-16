use async_trait::async_trait;

use crate::stash_record::StashRecord;

#[async_trait]
pub trait Sink {
    /// Handles processing a slice of `StashRecord`.
    async fn handle(
        &mut self,
        payload: &[StashRecord],
    ) -> Result<usize, Box<dyn std::error::Error>>;

    /// Sinks can be stateful and so want to be flushed upon graceful shutdown.
    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
