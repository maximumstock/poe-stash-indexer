use async_trait::async_trait;
use diesel::QueryResult;

use crate::stash_record::StashRecord;

#[async_trait]
pub trait Sink {
    /// Handles processing a slice of `StashRecord`.
    async fn handle(&self, payload: &[StashRecord]) -> Result<usize, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait SinkResume {
    /// Returns the next chunk id to continue from counting chunks of `StashTabResponse`.
    async fn get_next_chunk_id(&self) -> QueryResult<usize>;
    /// Returns the next change id to continue from based on previously fetched data.
    async fn get_next_change_id(&self) -> QueryResult<String>;
}
