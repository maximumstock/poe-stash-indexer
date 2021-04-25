-- Your SQL goes here
ALTER TABLE stash_records
ADD tick BIGINT NOT NULL DEFAULT -1;
CREATE INDEX stash_records_tick_idx on stash_records(tick);
