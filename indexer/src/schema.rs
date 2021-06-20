table! {
    stash_records (id) {
        created_at -> Timestamp,
        change_id -> Text,
        next_change_id -> Text,
        stash_id -> Text,
        stash_type -> Text,
        items -> Jsonb,
        public -> Bool,
        account_name -> Nullable<Text>,
        last_character_name -> Nullable<Text>,
        stash_name -> Nullable<Text>,
        league -> Nullable<Text>,
        id -> Int8,
        chunk_id -> Int8,
    }
}
