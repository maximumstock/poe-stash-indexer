/// Internal types that model what the data coming from PoE's API looks like.
pub mod protocol {
    use serde::{Deserialize, Serialize};

    /// The official API schema for the Publish Stash Tab API provided by GGG.
    ///
    /// See https://www.pathofexile.com/developer/docs/reference#publicstashes.
    ///
    /// Every individual API response refers to a specific change id and holds a list of
    /// stashes with their latest contents.
    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    pub struct PublicStashTabResponse {
        pub next_change_id: String,
        pub stashes: Vec<PublicStashChange>,
    }

    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    /// The official API schema for the changes of a single stash.
    ///
    /// See https://www.pathofexile.com/developer/docs/reference#type-PublicStashChange
    pub struct PublicStashChange {
        /// A unique 64 digit hexadecimal string
        pub id: String,
        /// If `false` then optional properties will be empty
        pub public: bool,
        #[serde(rename(deserialize = "accountName"))]
        pub account_name: Option<String>,
        pub stash: Option<String>,
        #[serde(rename(deserialize = "stashType"))]
        pub stash_type: String,
        pub league: Option<String>,
        pub items: Vec<Item>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    pub struct Item {
        /// Always `poe2` if present
        pub realm: Option<String>,
        pub verified: bool,
        pub w: u8,
        pub h: u8,
        pub icon: String,
        pub support: Option<bool>,
        #[serde(rename(deserialize = "stackSize"))]
        pub stack_size: Option<u16>,
        #[serde(rename(deserialize = "maxStackSize"))]
        pub max_stack_size: Option<u16>,
        #[serde(rename(deserialize = "stackSizeText"))]
        pub stack_size_text: Option<String>,
        pub league: Option<String>,
        pub id: Option<String>,
        #[serde(rename(deserialize = "unidentifiedTier"))]
        pub unidentified_tier: Option<u8>,
        pub influences: Option<serde_json::Value>,
        pub elder: Option<bool>,
        pub shaper: Option<bool>,
        pub searing: Option<bool>,
        pub tangled: Option<bool>,
        #[serde(rename(deserialize = "memoryItem"))]
        pub memory_item: Option<bool>,
        #[serde(rename(deserialize = "abyssJewel"))]
        pub abyss_jewel: Option<bool>,
        pub delve: Option<bool>,
        pub fractured: Option<bool>,
        pub synthesised: Option<bool>,
        pub sockets: Option<Vec<ItemSocket>>,
        #[serde(rename(deserialize = "socketedItems"))]
        pub socketed_items: Option<Vec<Item>>,
        pub name: String,
        #[serde(rename(deserialize = "typeLine"))]
        pub type_line: String,
        #[serde(rename(deserialize = "baseType"))]
        pub base_type: String,
        /// Normal, Magic, Rare, or Unique
        pub rarity: Option<String>,
        pub identified: bool,
        #[serde(rename(deserialize = "itemLevel"))]
        pub item_level: Option<u8>,
        pub ilvl: u8,
        pub note: Option<String>,
        #[serde(rename(deserialize = "forumNote"))]
        pub forum_note: Option<String>,
        #[serde(rename(deserialize = "lockedToCharacter"))]
        pub locked_to_character: Option<bool>,
        #[serde(rename(deserialize = "lockedToAccount"))]
        pub locked_to_account: Option<bool>,
        pub duplicated: Option<bool>,
        pub split: Option<bool>,
        pub corrupted: Option<bool>,
        pub unmodifiable: Option<bool>,
        #[serde(rename(deserialize = "cisRaceReward"))]
        pub cis_race_reward: Option<bool>,
        #[serde(rename(deserialize = "seaRaceReward"))]
        pub sea_race_reward: Option<bool>,
        #[serde(rename(deserialize = "thRaceReward"))]
        pub th_race_reward: Option<bool>,
        pub properties: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "notableProperties"))]
        pub notable_properties: Option<Vec<ItemProperty>>,
        pub requirements: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "weaponRequirements"))]
        pub weapon_properties: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "supportGemRequirements"))]
        pub support_gem_requirements: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "additionalRequirements"))]
        pub additional_requirements: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "nextLevelRequirements"))]
        pub next_level_requirements: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "grantedSkills"))]
        pub granted_skills: Option<Vec<ItemProperty>>,
        #[serde(rename(deserialize = "talismanTier"))]
        pub talisman_tier: Option<u8>,
        pub rewards: Option<Vec<ItemReward>>,
        #[serde(rename(deserialize = "secDescrText"))]
        pub sec_descr_text: Option<String>,
        #[serde(rename(deserialize = "utilityMods"))]
        pub utility_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "logbookMods"))]
        pub logbook_mods: Option<Vec<LogbookMods>>,
        #[serde(rename(deserialize = "enchantMods"))]
        pub enchant_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "runeMods"))]
        pub rune_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "scourgeMods"))]
        pub scourge_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "implicitMods"))]
        pub implicit_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "ultimatumMods"))]
        pub ultimatum_mods: Option<Vec<UltimatumMod>>,
        #[serde(rename(deserialize = "explicitMods"))]
        pub explicit_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "craftedMods"))]
        pub crafted_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "fracturedMods"))]
        pub fractured_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "crucibleMods"))]
        pub crucible_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "cosmeticMods"))]
        pub cosmetic_mods: Option<Vec<String>>,
        #[serde(rename(deserialize = "veiledMods"))]
        pub veiled_mods: Option<Vec<String>>,
        pub veiled: Option<bool>,
        #[serde(rename(deserialize = "descrText"))]
        pub descr_text: Option<String>,
        #[serde(rename(deserialize = "flavourText"))]
        pub flavour_text: Option<Vec<String>>,
        #[serde(rename(deserialize = "flavourTextParsed"))]
        pub flavour_text_parsed: Option<Vec<String>>,
        #[serde(rename(deserialize = "flavourTextNote"))]
        pub flavour_text_note: Option<String>,
        #[serde(rename(deserialize = "prophecyText"))]
        pub prophecy_text: Option<String>,
        #[serde(rename(deserialize = "isRelic"))]
        pub is_relic: Option<bool>,
        #[serde(rename(deserialize = "foilVariation"))]
        pub foil_variation: Option<u8>,
        pub replica: Option<bool>,
        pub foreseeing: Option<bool>,
        #[serde(rename(deserialize = "incubatedItem"))]
        pub incubated_item: Option<IncubatedItem>,
        pub scourged: Option<ScourgedItem>,
        pub crucible: Option<CrucibleItem>,
        pub ruthless: Option<bool>,
        #[serde(rename(deserialize = "frameType"))]
        pub frame_type: Option<u8>,
        #[serde(rename(deserialize = "artFilename"))]
        pub art_filename: Option<String>,
        pub hybrid: Option<HybridItem>,
        pub extended: Option<ItemExtendedProp>,
        pub x: Option<u8>,
        pub y: Option<u8>,
        #[serde(rename(deserialize = "inventoryId"))]
        pub inventory_id: Option<String>,
        pub socket: Option<u8>,
        pub colour: Option<String>,

        /// PoE 2 only - not yet filled
        #[serde(rename(deserialize = "getSockets"))]
        pub gem_sockets: Option<Vec<String>>,
        #[serde(rename(deserialize = "gemTabs"))]
        pub gem_tabs: Option<Vec<GemTab>>,
        #[serde(rename(deserialize = "gemBackground"))]
        pub gem_background: Option<String>,
        #[serde(rename(deserialize = "gemSkill"))]
        pub gem_skill: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ItemExtendedProp {
        pub prefixes: Option<u8>,
        pub suffixes: Option<u8>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ItemSocket {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ItemProperty {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ItemReward {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct LogbookMods {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct UltimatumMod {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct GemTab {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct IncubatedItem {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct ScourgedItem {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct CrucibleItem {}

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub struct HybridItem {}
}
