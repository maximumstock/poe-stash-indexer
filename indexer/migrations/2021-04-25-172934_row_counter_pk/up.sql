-- Your SQL goes here
-- Remove old primary key
ALTER TABLE stash_records DROP CONSTRAINT IF EXISTS stash_records_pkey;
-- Add new id column
ALTER TABLE stash_records
ADD id BIGINT GENERATED ALWAYS AS IDENTITY;
-- Move primary key constraint
ALTER TABLE stash_records
ADD CONSTRAINT stash_records_pk PRIMARY KEY (id);
-- Add unique constraint on previous primary key
ALTER TABLE stash_records
ADD CONSTRAINT stash_records_change_id_stash_id_unique UNIQUE (change_id, stash_id);
