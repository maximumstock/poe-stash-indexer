use std::collections::{HashMap, HashSet, VecDeque};

use sqlx::{Pool, Postgres};

use crate::stash::StashRecord;

pub struct StashRecordIterator<'a> {
    pool: &'a Pool<Postgres>,
    runtime: &'a tokio::runtime::Runtime,
    league: &'a str,
    page_size: i64,
    page: (i64, i64),
    buffer: VecDeque<StashRecord>,
    available_chunks: usize,
}

impl<'a> StashRecordIterator<'a> {
    pub fn new(
        pool: &'a Pool<Postgres>,
        runtime: &'a tokio::runtime::Runtime,
        page_size: i64,
        league: &'a str,
    ) -> Self {
        Self {
            pool,
            runtime,
            league,
            page_size,
            page: (0, page_size),
            buffer: VecDeque::new(),
            available_chunks: 0,
        }
    }

    fn needs_data(&self) -> bool {
        self.available_chunks < 2
    }

    fn count_available_chunks(&self) -> usize {
        self.buffer
            .iter()
            .map(|i| &i.change_id)
            .collect::<HashSet<_>>()
            .len()
    }

    fn load_data(&mut self) -> Result<(), sqlx::Error> {
        let next_page = self.runtime.block_on(fetch_stash_records_paginated(
            self.pool,
            self.page.0,
            self.page.1,
            self.league,
        ))?;

        self.buffer.extend(next_page);
        self.page = (self.page.1, self.page.1 + self.page_size);
        self.available_chunks = self.count_available_chunks();
        Ok(())
    }

    fn extract_first_chunk(&mut self) -> Vec<StashRecord> {
        if self.needs_data() {
            panic!("Expected to have more data");
        }

        let next_change_id = &self
            .buffer
            .front()
            .expect("No data where some was expected")
            .change_id
            .clone();

        // TODO can we allocate this just once?
        let mut chunk = vec![];

        while let Some(next) = self.buffer.front() {
            if next.change_id.eq(next_change_id) {
                let v = self
                    .buffer
                    .pop_front()
                    .expect("taking first stash record from queue");
                chunk.push(v);
            } else {
                break;
            }
        }

        self.available_chunks -= 1;

        chunk
    }

    pub fn next_chunk(&mut self) -> Option<Vec<StashRecord>> {
        while self.needs_data() {
            self.load_data().expect("Fetching next page failed");
        }

        Some(self.extract_first_chunk())
    }
}

async fn fetch_stash_records_paginated(
    pool: &Pool<Postgres>,
    start: i64,
    end: i64,
    league: &str,
) -> Result<Vec<StashRecord>, sqlx::Error> {
    sqlx::query_as::<_, StashRecord>(
        "SELECT change_id, next_change_id, stash_id, account_name, league, items, created_at
             FROM stash_records
             WHERE league = $1 and int8range($2, $3, '[]') @> int8range(id, id, '[]')",
    )
    .bind(league)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}
