use async_trait::async_trait;
use diesel::QueryResult;

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

#[async_trait]
pub trait SinkResume {
    /// Returns the next change id to continue from based on previously fetched data.
    async fn get_next_change_id(&self) -> QueryResult<String>;
}
