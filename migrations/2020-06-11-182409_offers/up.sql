-- Your SQL goes here
CREATE TABLE offers (
  id BIGSERIAL PRIMARY KEY,
  sell TEXT NOT NULL,
  buy TEXT NOT NULL,
  conversion_rate REAL NOT NULL,
  stock BIGINT NOT NULL,
  league TEXT,
  account_name TEXT,
  category TEXT,
  public BOOLEAN NOT NULL,
  stash_type TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  change_id TEXT NOT NULL
)
