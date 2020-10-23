-- Your SQL goes here
CREATE TABLE offers (
  id BIGSERIAL PRIMARY KEY,
  sell TEXT NOT NULL,
  buy TEXT NOT NULL,
  conversion_rate REAL NOT NULL,
  stock BIGINT NOT NULL,
  league TEXT,
  account_name TEXT,
  item_id TEXT NOT NULL,
  stash_id TEXT NOT NULL,
  stash_name TEXT,
  category TEXT,
  public BOOLEAN NOT NULL,
  stash_type TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  change_id TEXT NOT NULL
);
CREATE INDEX offers_sell_idx on offers(sell);
CREATE INDEX offers_buy_idx on offers(buy);
CREATE INDEX offers_league_idx on offers(league);
CREATE INDEX offers_accountname_idx on offers(account_name);
CREATE INDEX offers_category_idx on offers(category);
CREATE INDEX offers_change_id_idx on offers (change_id);
