use diesel::QueryResult;

use crate::stash_record::StashRecord;

pub trait Sink {
    /// Handles processing a slice of `StashRecord`.
    fn handle(&self, payload: &[StashRecord]) -> Result<usize, Box<dyn std::error::Error>>;
}

pub trait SinkResume {
    /// Returns the next chunk id to continue from counting chunks of `StashTabResponse`.
    fn get_next_chunk_id(&self) -> QueryResult<Option<i64>>;
    /// Returns the next change id to continue from based on previously fetched data.
    fn get_next_change_id(&self) -> QueryResult<String>;
}

// pub struct UniSink {
//     inner: Vec<dyn Sink>,
// }

// impl UniSink {
//     fn builder() -> UniSinkBuilder {
//         UniSinkBuilder::default()
//     }
// }

// #[derive(Default)]
// pub struct UniSinkBuilder<'a> {
//     database_url: Option<&'a str>,
//     rabbitmq_url: Option<&'a str>,
// }

// impl UniSinkBuilder {
//     fn with_database_sink(self, database_url: &'a str) -> Self {
//         self.database_url = Some(database_url);
//     }

//     fn with_rabbitmq_sink(self, rabbitmq_url: &'a str) -> Self {
//         self.database_url = Some(rabbitmq_url);
//     }

//     fn build(self) -> UniSink {
//         UniSink {
//             inner: ve
//         }
//     }
// }
