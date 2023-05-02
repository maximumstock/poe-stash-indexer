use async_trait::async_trait;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};

use crate::schema::stash_records::dsl::*;
use crate::stash_record::StashRecord;

use super::sink::Sink;

pub struct Postgres {
    pool: Pool<AsyncPgConnection>,
}

impl Postgres {
    pub async fn connect(database_url: &str) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        Postgres {
            pool: Pool::builder()
                .build(config)
                .await
                .expect("Postgres database connect"),
        }
    }
}

#[async_trait]
impl Sink for Postgres {
    #[tracing::instrument(skip(self, records), name = "handle-postgres")]
    async fn handle(&self, records: &[StashRecord]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get().await?;

        let query = diesel::insert_into(stash_records).values(records).into();

        diesel::insert_into(stash_records)
            .values(records)
            .execute(&mut conn)
            .await
            .map_err(|e| e.into())
    }
}

// #[async_trait]
// impl SinkResume for Postgres {
//     #[tracing::instrument(skip(self))]
//     fn get_next_chunk_id(&self) -> QueryResult<Option<i64>> {
//         stash_records
//             .select(max(chunk_id))
//             .first::<Option<i64>>(&self.conn)
//     }

//     #[tracing::instrument(skip(self))]
//     fn get_next_change_id(&self) -> QueryResult<String> {
//         let latest_created_at = stash_records
//             .select(max(created_at))
//             .first::<Option<NaiveDateTime>>(&self.conn);

//         match latest_created_at {
//             Ok(Some(last_created_at)) => stash_records
//                 .select(next_change_id)
//                 .filter(created_at.eq(&last_created_at))
//                 .first(&self.conn),
//             Ok(None) => Err(diesel::result::Error::NotFound),
//             Err(e) => Err(e),
//         }
//     }
// }
