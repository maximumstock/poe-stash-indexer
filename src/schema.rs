table! {
    offers (id) {
        id -> Int8,
        sell -> Text,
        buy -> Text,
        conversion_rate -> Float4,
        stock -> Int8,
        league -> Nullable<Text>,
        account_name -> Nullable<Text>,
        item_id -> Text,
        stash_id -> Text,
        stash_name -> Nullable<Text>,
        category -> Nullable<Text>,
        public -> Bool,
        stash_type -> Text,
        created_at -> Timestamp,
        change_id -> Text,
    }
}
