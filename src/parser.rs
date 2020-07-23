use crate::schema::offers;
use diesel::Insertable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct StashTabResponse {
    pub next_change_id: String,
    stashes: Vec<Stash>,
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

#[derive(Debug, PartialEq, Serialize, Insertable)]
#[table_name = "offers"]
pub struct Offer {
    sell: String,
    buy: String,
    conversion_rate: f32,
    stock: Option<i64>,
    league: Option<String>,
    account_name: Option<String>,
    category: String,
    public: bool,
    stash_type: String,
    created_at: std::time::SystemTime,
    change_id: String,
}

#[derive(Debug, PartialEq)]
pub enum ItemParseError {
    Price(String),
    UnknownNoteFormat(String),
}

#[derive(Debug, PartialEq)]
pub enum ItemParseResult {
    Success(Offer),
    Error(ItemParseError),
    Empty,
}

fn parse_price(input: &str) -> Result<f32, ItemParseError> {
    if input.contains('/') {
        let parts = input.split('/').collect::<Vec<_>>();
        let numerator = parts[0].parse::<f32>();
        let denominator = parts[1].parse::<f32>();
        match (numerator, denominator) {
            (Ok(a), Ok(b)) => Ok(a / b),
            (_, _) => Err(ItemParseError::Price(input.to_owned())),
        }
    } else {
        input
            .parse()
            .map_err(|_| ItemParseError::Price(input.to_owned()))
    }
}

fn is_note_match(input: &str) -> bool {
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"[\d\.\d/]+[\s]+([a-zA-Z-_]+)$").unwrap();
    }
    RE.is_match(input)
}

fn parse_note(input: &str) -> Result<Note, ItemParseError> {
    if !is_note_match(&input) {
        return Err(ItemParseError::UnknownNoteFormat(input.to_owned()));
    }

    let parts = input.split_whitespace().collect::<Vec<_>>();

    if parts.len() >= 3 {
        Ok(Note {
            price: parse_price(parts.get(parts.len() - 2).unwrap())?,
            currency_id: String::from(parts.last().unwrap().to_owned()),
        })
    } else {
        Err(ItemParseError::UnknownNoteFormat(input.to_owned()))
    }
}

fn parse_item(
    item: &Item,
    stash: &Stash,
    id: &str,
    created_at: std::time::SystemTime,
) -> ItemParseResult {
    if item.note.is_none() || !item.name.is_empty() || item.stack_size.is_none() {
        ItemParseResult::Empty
    } else {
        match parse_note(item.note.clone().unwrap().as_ref()) {
            Ok(note) => ItemParseResult::Success(Offer {
                sell: item.extended.base_type.clone(),
                buy: note.currency_id,
                conversion_rate: note.price,
                stock: item.stack_size.map(|i| i as i64),
                account_name: stash.account_name.clone(),
                league: stash.league.clone(),
                public: stash.public,
                category: item.extended.category.clone(),
                stash_type: stash.stash_type.clone(),
                change_id: id.to_owned(),
                created_at,
            }),
            Err(e) => ItemParseResult::Error(e),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Note {
    price: f32,
    currency_id: String,
}

pub fn parse_items(response: &StashTabResponse, id: &str) -> Vec<ItemParseResult> {
    let mut results = vec![];
    let created_at = std::time::SystemTime::now();

    for stash in &response.stashes {
        for item in &stash.items {
            let parsed = parse_item(item, stash, id, created_at);
            results.push(parsed);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_single_integers() {
        assert!(parse_price("1").unwrap().eq(&1_f32));
    }

    #[test]
    fn test_parse_price_single_floats() {
        assert!(parse_price("123.2").unwrap().eq(&123.2_f32));
        assert!(parse_price(".2").unwrap().eq(&0.2_f32));
    }

    #[test]
    fn test_parse_price_fractions() {
        assert!(parse_price("70/20").unwrap().eq(&3.5_f32));
        assert!(parse_price("7.0/2.0").unwrap().eq(&3.5_f32));
        assert!(parse_price("7/2.0").unwrap().eq(&3.5_f32));
        assert!(parse_price("7.0/2").unwrap().eq(&3.5_f32));
    }

    #[test]
    fn test_parse_price_invalid_cases() {
        assert_eq!(
            parse_price("5/"),
            Err(ItemParseError::Price(String::from("5/")))
        );
        assert_eq!(
            parse_price("/"),
            Err(ItemParseError::Price(String::from("/")))
        );
        assert_eq!(
            parse_price("30:8"),
            Err(ItemParseError::Price(String::from("30:8")))
        );
        assert_eq!(
            parse_price("7.0/2,09"),
            Err(ItemParseError::Price(String::from("7.0/2,09")))
        );
    }

    #[test]
    fn test_is_note_match_prefix_symbol() {
        assert!(is_note_match("~price 20 chaos"));
        assert!(is_note_match("price 20 chaos"));
        assert!(is_note_match("-price 20 chaos"));
    }

    #[test]
    fn test_is_note_match_prefix() {
        assert!(is_note_match("price 20 chaos"));
        assert!(is_note_match("buyout 20 chaos"));
        assert!(is_note_match("bo 20 chaos"));
        assert!(is_note_match("b/o 20 chaos"));
        assert!(is_note_match("20 chaos"));
    }

    #[test]
    fn test_is_note_match_longer_notes() {
        assert!(is_note_match("01/01/20 ~price .2 dense-fossil"));
        assert!(is_note_match("gibberish ~price .2 dense-fossil"));
    }

    #[test]
    fn test_is_note_match_invalid_cases() {
        assert!(!is_note_match("~price  dense-fossil"));
    }

    #[test]
    fn test_parse_note() {
        assert_eq!(
            parse_note("~price 10 chaos").unwrap(),
            Note {
                price: 10.0,
                currency_id: String::from("chaos")
            }
        );
        assert_eq!(
            parse_note("~b/o 10.0 chaos").unwrap(),
            Note {
                price: 10.0,
                currency_id: String::from("chaos")
            }
        );
        assert_eq!(
            parse_note("~b/o 1000.0/100.0 chaos").unwrap(),
            Note {
                price: 10.0,
                currency_id: String::from("chaos")
            }
        );
        assert_eq!(
            parse_note("~b/o  chaos"),
            Err(ItemParseError::UnknownNoteFormat(String::from(
                "~b/o  chaos"
            )))
        );
        assert_eq!(
            parse_note("~b/o 1/1 Chaos Orb"),
            Err(ItemParseError::UnknownNoteFormat(String::from(
                "~b/o 1/1 Chaos Orb"
            )))
        );
    }
}
