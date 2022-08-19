-- Add migration script here

CREATE INDEX IF NOT EXISTS hc_stash_id_idx on challenge_hc (stash_id);
CREATE INDEX IF NOT EXISTS hc_sell_idx on challenge_hc (sell);
CREATE INDEX IF NOT EXISTS hc_buy_idx on challenge_hc (buy);