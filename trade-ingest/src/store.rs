use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};




use crate::{note_parser::PriceParser, source::StashRecord};

type StashId = String;
type ItemId = String;
type OfferIndex = u64;

#[derive(Debug, Serialize, Deserialize)]
/// Describes an offer from the view of the seller.
pub struct Offer {
    pub(crate) item_id: ItemId,
    pub(crate) stash_id: StashId,
    /// Item that is sold from the point of view of the seller.
    pub(crate) sell: String,
    /// Item that the seller receives.
    pub(crate) buy: String,
    pub(crate) seller_account: String,
    pub(crate) stock: u32,
    pub(crate) conversion_rate: f32,
    pub(crate) created_at: u64,
}

impl From<StashRecord> for Vec<Offer> {
    fn from(stash: StashRecord) -> Self {
        let account_name = stash.account_name;
        let stash_id = stash.stash_id;
        let price_parser = PriceParser::new();

        stash
            .items
            .into_iter()
            .filter(|item| item.note.is_some())
            .filter_map(|item| {
                if let Ok(price) = price_parser.parse_price(&item.note.unwrap()) {
                    let sold_item_name = match item.name.as_str() {
                        "" => item.type_line,
                        _ => item.name,
                    };

                    Some(Offer {
                        stock: item.stack_size.unwrap_or(1),
                        sell: sold_item_name,
                        conversion_rate: price.ratio,
                        buy: price.item.to_owned(),
                        item_id: item.id,
                        seller_account: account_name.clone(),
                        stash_id: stash_id.clone(),
                        created_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .expect("Failed to create timestamp")
                            .as_secs(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl PartialEq for Offer {
    fn eq(&self, other: &Self) -> bool {
        self.item_id.eq(&other.item_id) && self.stash_id.eq(&other.stash_id)
    }
}

impl Eq for Offer {}

impl Hash for Offer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.item_id.hash(state);
        self.stash_id.hash(state);
    }
}

#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}
