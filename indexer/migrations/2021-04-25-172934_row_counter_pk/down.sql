-- This file should undo anything in `up.sql`
-- Reverse: Add unique constraint on previous primary key
ALTER TABLE stash_records DROP CONSTRAINT stash_records_change_id_stash_id_unique;
-- Reverse: Move primary key constraint
ALTER TABLE stash_records DROP CONSTRAINT stash_records_pk;
ALTER TABLE stash_records
ADD CONSTRAINT stash_records_pkey PRIMARY KEY (change_id, stash_id);
-- Reverse: Add new id column
ALTER TABLE stash_records DROP id;
