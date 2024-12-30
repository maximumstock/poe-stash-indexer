-- Add migration script here
CREATE INDEX IF NOT EXISTS stash_id_idx on challenge (stash_id);
CREATE INDEX IF NOT EXISTS sell_idx on challenge (sell);
CREATE INDEX IF NOT EXISTS buy_idx on challenge (buy);
