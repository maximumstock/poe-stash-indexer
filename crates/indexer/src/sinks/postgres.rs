use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::{dsl::max, ExpressionMethods, QueryDsl, QueryResult};
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};

use crate::schema::stash_records::dsl::*;
use crate::stash_record::StashRecord;

use super::sink::{Sink, SinkResume};

pub struct PostgresSink {
    pool: Pool<AsyncPgConnection>,
}

impl PostgresSink {
    pub async fn connect(database_url: &str) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        PostgresSink {
            pool: Pool::builder()
                .build(config)
                .await
                .expect("Postgres database connect"),
        }
    }
}

#[async_trait]
impl Sink for PostgresSink {
    #[tracing::instrument(skip(self, records), name = "sink-handle-postgres")]
    async fn handle(
        &mut self,
        records: &[StashRecord],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get().await?;

        diesel::insert_into(stash_records)
            .values(records)
            .execute(&mut conn)
            .await
            .map_err(|e| e.into())
    }
}

#[async_trait]
impl SinkResume for PostgresSink {
    #[tracing::instrument(skip(self))]
    async fn get_next_change_id(&self) -> QueryResult<String> {
        let mut conn = self.pool.get().await.unwrap();

        let latest_created_at = stash_records
            .select(max(created_at))
            .first::<Option<NaiveDateTime>>(&mut conn)
            .await?;

        match latest_created_at {
            Some(last_created_at) => {
                stash_records
                    .select(next_change_id)
                    .filter(created_at.eq(&last_created_at))
                    .first(&mut conn)
                    .await
            }
            None => Err(diesel::result::Error::NotFound),
        }
    }
}
