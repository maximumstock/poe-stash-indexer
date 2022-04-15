-- Add migration script here
CREATE TABLE IF NOT EXISTS challenge (
    item_id text NOT NULL,
    stash_id text NOT NULL,
    seller_account text NOT NULL,
    stock int NOT NULL,
    sell text NOT NULL,
    buy text NOT NULL,
    conversion_rate real NOT NULL,
    created_at timestamp NOT NULL
);

CREATE TABLE IF NOT EXISTS challenge_hc (
    item_id text NOT NULL,
    stash_id text NOT NULL,
    seller_account text NOT NULL,
    stock int NOT NULL,
    sell text NOT NULL,
    buy text NOT NULL,
    conversion_rate real NOT NULL,
    created_at timestamp NOT NULL
);