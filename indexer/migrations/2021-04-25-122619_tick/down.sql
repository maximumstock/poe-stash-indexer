-- This file should undo anything in `up.sql`
ALTER TABLE stash_records DROP tick;
DROP INDEX IF EXISTS stash_records_tick_idx;
