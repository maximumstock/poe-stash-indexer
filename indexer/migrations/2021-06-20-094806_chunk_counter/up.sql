-- Your SQL goes here
ALTER TABLE stash_records
ADD chunk_id BIGINT NOT NULL DEFAULT 0;
CREATE INDEX stash_records_chunk_id_idx on stash_records (chunk_id);
