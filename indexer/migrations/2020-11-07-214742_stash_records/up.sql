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
  PRIMARY KEY (change_id, stash_id)
);
CREATE INDEX stash_records_change_id_idx on stash_records(change_id);
CREATE INDEX stash_records_next_change_id_idx on stash_records(next_change_id);
CREATE INDEX stash_records_stash_id_idx on stash_records(stash_id);
CREATE INDEX stash_records_stash_type_idx on stash_records(stash_type);
CREATE INDEX stash_records_public_idx on stash_records(public);
CREATE INDEX stash_records_account_name_idx on stash_records(account_name);
CREATE INDEX stash_records_league_idx on stash_records(league);
