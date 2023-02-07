-- Your SQL goes here
CREATE TABLE stash_records (
  created_at TIMESTAMP NOT NULL,
  change_id TEXT NOT NULL,
  next_change_id TEXT NOT NULL,
  stash_id TEXT NOT NULL,
  stash_type TEXT NOT NULL,
  items JSONB NOT NULL,
  public BOOLEAN NOT NULL,
  account_name TEXT,
  last_character_name TEXT,
  stash_name TEXT,
  league TEXT,
  PRIMARY KEY (created_at)
);

-- Chunk Time Interval of 12 hours
SELECT create_hypertable('stash_records', 'created_at', chunk_time_interval => '12 hours'::interval);