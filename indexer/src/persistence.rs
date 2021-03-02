use diesel::{Connection, PgConnection, QueryDsl, QueryResult, RunQueryDsl};

use crate::diesel::ExpressionMethods;
use crate::schema::stash_records::dsl::*;
use crate::StashRecord;

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
            conn: PgConnection::establish(&database_url).expect("lul"),
        }
    }

    pub fn get_next_change_id(&self) -> QueryResult<String> {
        stash_records
            .select(next_change_id)
            .order(change_id.desc())
            .first::<String>(&self.conn)
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

// pub struct CSVLog<'a> {
//     filepath: &'a str,
// }

// impl<'a> CSVLog<'a> {
//     pub fn new(filepath: &'a str) -> Self {
//         Self { filepath }
//     }
// }

// fn prepare_file(filepath: &str) -> Result<std::fs::File, Box<dyn std::error::Error>> {
//     std::fs::OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(&filepath)
//         .map_err(|e| e.into())
// }

// impl Persist for CSVLog<'_> {
//     fn save_offers(&self, offers: &[Offer]) -> PersistResult {
//         let file = prepare_file(&self.filepath)?;
//         let mut writer = csv::Writer::from_writer(BufWriter::new(file));
//         for o in offers {
//             writer.serialize(o)?;
//         }
//         writer.flush()?;
//         Ok(offers.len())
//     }
// }
