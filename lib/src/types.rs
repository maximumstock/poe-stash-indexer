use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct StashTabResponse {
    pub next_change_id: String,
    pub stashes: Vec<Stash>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Stash {
    #[serde(rename(deserialize = "accountName"))]
    account_name: Option<String>,
    #[serde(rename(deserialize = "lastCharacterName"))]
    last_character_name: Option<String>,
    id: String,
    stash: Option<String>,
    #[serde(rename(deserialize = "stashType"))]
    stash_type: String,
    items: Vec<Item>,
    public: bool,
    league: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Item {
    name: String,
    id: String,
    #[serde(rename(deserialize = "inventoryId"))]
    inventory_name: Option<String>,
    note: Option<String>,
    #[serde(rename(deserialize = "typeLine"))]
    type_line: String,
    #[serde(rename(deserialize = "stackSize"))]
    stack_size: Option<u32>,
    extended: ItemExtendedProp,
}

#[derive(Debug, Deserialize, Clone)]
struct ItemExtendedProp {
    category: String,
    #[serde(rename(deserialize = "baseType"))]
    base_type: String,
}
