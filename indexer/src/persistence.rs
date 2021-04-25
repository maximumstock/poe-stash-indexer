use chrono::NaiveDateTime;
use diesel::{dsl::max, Connection, PgConnection, QueryDsl, QueryResult, RunQueryDsl};

use crate::schema::stash_records::dsl::*;
use crate::StashRecord;
use crate::{diesel::ExpressionMethods, schema};

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

    pub fn get_latest_created_at(&self) -> QueryResult<Option<NaiveDateTime>> {
        use schema::stash_records::dsl::*;

        stash_records
            .select(max(created_at))
            .first::<Option<NaiveDateTime>>(&self.conn)
    }

    pub fn get_next_change_id(&self) -> QueryResult<String> {
        if let Ok(Some(last_created_at)) = self.get_latest_created_at() {
            stash_records
                .select(next_change_id)
                .filter(created_at.eq(&last_created_at))
                .first(&self.conn)
        } else {
            Err(diesel::result::Error::NotFound)
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
