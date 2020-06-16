use super::parser::Offer;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::fs::File;
use std::io::Write;

pub trait Persist {
    // TODO return Result
    fn save_offers(&self, offers: &Vec<Offer>) -> ();
}
// by default in a dev environment
pub struct FileDb {}
impl Persist for FileDb {
    fn save_offers(&self, offers: &Vec<Offer>) -> () {
        let mut file = File::create("data/db.json").unwrap();
        let output = serde_json::to_string_pretty(offers).unwrap();
        file.write(output.as_ref()).expect("WRITE failed...");
    }
}
// by default in a prod environment
pub struct PgDb<'a> {
    conn: &'a PgConnection,
}
impl<'a> PgDb<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        PgDb { conn: connection }
    }
}
impl Persist for PgDb<'_> {
    fn save_offers(&self, offers: &Vec<Offer>) -> () {
        diesel::insert_into(super::schema::offers::table)
            .values(offers)
            .execute(self.conn)
            .expect("INSERT failed...");
    }
}
