use chrono::NaiveDateTime;
use diesel::{dsl::max, Connection, PgConnection, QueryDsl, QueryResult, RunQueryDsl};

use crate::diesel::ExpressionMethods;
use crate::schema::stash_records::dsl::*;
use crate::stash_record::StashRecord;

use super::sink::{Sink, SinkResume};

pub struct Postgres {
    conn: PgConnection,
}

impl Postgres {
    pub fn connect(database_url: &str) -> Self {
        Postgres {
            conn: PgConnection::establish(database_url).expect("Could not connect to database"),
        }
    }
}

impl Sink for Postgres {
    fn handle(&self, records: &[StashRecord]) -> Result<usize, Box<dyn std::error::Error>> {
        diesel::insert_into(stash_records)
            .values(records)
            .on_conflict_do_nothing()
            .execute(&self.conn)
            .map_err(|e| e.into())
    }
}

impl SinkResume for Postgres {
    fn get_next_chunk_id(&self) -> QueryResult<Option<i64>> {
        stash_records
            .select(max(chunk_id))
            .first::<Option<i64>>(&self.conn)
    }

    fn get_next_change_id(&self) -> QueryResult<String> {
        let latest_created_at = stash_records
            .select(max(created_at))
            .first::<Option<NaiveDateTime>>(&self.conn);

        match latest_created_at {
            Ok(Some(last_created_at)) => stash_records
                .select(next_change_id)
                .filter(created_at.eq(&last_created_at))
                .first(&self.conn),
            Ok(None) => Err(diesel::result::Error::NotFound),
            Err(e) => Err(e),
        }
    }
}
