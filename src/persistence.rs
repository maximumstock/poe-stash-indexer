use super::parser::Offer;
use super::schema::offers;
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub struct PgDb {
    conn: PgConnection,
}
impl PgDb {
    pub fn new(database_url: &str) -> Self {
        PgDb {
            conn: PgConnection::establish(&database_url).expect("lul"),
        }
    }

    pub fn save_offers(&self, offers: &[Offer]) -> QueryResult<usize> {
        diesel::insert_into(super::schema::offers::table)
            .values(offers)
            .on_conflict_do_nothing()
            .execute(&self.conn)
    }

    pub fn get_last_read_change_id(&self) -> QueryResult<String> {
        offers::table
            .select(offers::change_id)
            .order(offers::created_at.desc())
            .first::<String>(&self.conn)
    }
}
