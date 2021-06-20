use chrono::NaiveDateTime;
use diesel::{dsl::max, Connection, PgConnection, QueryDsl, QueryResult, RunQueryDsl};

use crate::diesel::ExpressionMethods;
use crate::schema::stash_records::dsl::*;
use crate::stash_record::StashRecord;

type PersistResult = Result<usize, Box<dyn std::error::Error>>;

pub trait Persist {
    fn save(&self, payload: &[StashRecord]) -> PersistResult;
}

pub struct PgDb {
    conn: PgConnection,
}

impl PgDb {
    pub fn new(database_url: &str) -> Self {
        PgDb {
            conn: PgConnection::establish(&database_url).expect("Could not connect to database"),
        }
    }

    fn get_latest_created_at(&self) -> QueryResult<Option<NaiveDateTime>> {
        stash_records
            .select(max(created_at))
            .first::<Option<NaiveDateTime>>(&self.conn)
    }

    pub fn get_next_chunk_id(&self) -> QueryResult<Option<i64>> {
        stash_records
            .select(max(chunk_id))
            .first::<Option<i64>>(&self.conn)
    }

    pub fn get_next_change_id(&self) -> QueryResult<String> {
        match self.get_latest_created_at() {
            Ok(Some(last_created_at)) => stash_records
                .select(next_change_id)
                .filter(created_at.eq(&last_created_at))
                .first(&self.conn),
            Ok(None) => Err(diesel::result::Error::NotFound),
            Err(e) => Err(e),
        }
    }
}

impl Persist for PgDb {
    fn save(&self, records: &[StashRecord]) -> PersistResult {
        diesel::insert_into(stash_records)
            .values(records)
            .on_conflict_do_nothing()
            .execute(&self.conn)
            .map_err(|e| e.into())
    }
}
