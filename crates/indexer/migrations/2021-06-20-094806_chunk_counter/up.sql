-- Your SQL goes here
ALTER TABLE stash_records ADD chunk_id BIGINT DEFAULT NULL;

CREATE INDEX stash_records_chunk_id_idx on stash_records (chunk_id);
