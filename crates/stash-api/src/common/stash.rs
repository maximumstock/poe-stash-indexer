use chrono::NaiveDateTime;
use protocol::Item;
use serde::Serialize;

/// Internal types that model what the data coming from PoE's API looks like.
pub mod protocol {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    pub struct StashTabResponseInternal {
        pub next_change_id: String,
        pub stashes: Vec<StashInternal>,
    }

    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    /// Structure and types according to https://www.pathofexile.com/developer/docs/reference#type-PublicStashChange
    pub struct StashInternal {
        #[serde(rename(deserialize = "accountName"))]
        pub account_name: Option<String>,
        #[serde(rename(deserialize = "lastCharacterName"))]
        pub last_character_name: Option<String>,
        pub id: String,
        pub stash: Option<String>,
        #[serde(rename(deserialize = "stashType"))]
        pub stash_type: String,
        pub items: Vec<Item>,
        pub public: bool,
        pub league: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    pub struct Item {
        pub name: String,
        pub id: String,
        pub note: Option<String>,
        #[serde(rename(deserialize = "typeLine"))]
        pub type_line: String,
        #[serde(rename(deserialize = "stackSize"))]
        pub stack_size: Option<u32>,
        pub extended: ItemExtendedProp,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ItemExtendedProp {
        pub category: String,
        #[serde(rename(deserialize = "baseType"))]
        pub base_type: Option<String>,
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Stash {
    pub account_name: Option<String>,
    pub last_character_name: Option<String>,
    pub id: String,
    pub stash: Option<String>,
    pub stash_type: String,
    pub items: Vec<Item>,
    pub public: bool,
    pub league: Option<String>,
    pub created_at: NaiveDateTime,
    pub change_id: String,
    pub next_change_id: String,
}
