use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    fmt::Debug,
    hash::{Hash, Hasher},
    io::Write,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use crate::{assets::AssetIndex, league::League, note_parser::PriceParser, source::StashRecord};

type StashId = String;
type ItemId = String;
type OfferIndex = u64;

#[derive(Debug, Serialize, Deserialize)]
/// Describes an offer from the view of the seller.
pub struct Offer {
    item_id: ItemId,
    stash_id: StashId,
    /// Item that is sold from the point of view of the seller.
    sell: String,
    /// Item that the seller receives.
    buy: String,
    seller_account: String,
    stock: u32,
    conversion_rate: f32,
    created_at: u64,
}

impl Offer {
    pub fn get_index(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
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

#[derive(Debug, Hash, Clone)]
struct Conversion<'a> {
    sell: &'a str,
    buy: &'a str,
}

impl<'a> Conversion<'a> {
    pub fn new(sell: &'a str, buy: &'a str) -> Self {
        Self { sell, buy }
    }

    pub fn get_index(&self) -> ConversionIndex {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl<'a> From<&'a Offer> for Conversion<'a> {
    fn from(offer: &'a Offer) -> Self {
        Conversion {
            sell: &offer.sell,
            buy: &offer.buy,
        }
    }
}

type ConversionIndex = u64;

const STORE_BASE_PATH: &str = "/app/trade/trade-store";

#[derive(Debug, TypedBuilder, Eq, PartialEq, Serialize, Deserialize)]
pub struct Store {
    league: League,
    /// Holds _all_ offers for the given league indexed by a manually created hash.
    #[builder(default)]
    offers: HashMap<OfferIndex, Offer>,
    /// Holds an index mapping from Stash to Offer hashes to find offers from a given stash.
    #[builder(default)]
    stash_to_offers_idx: HashMap<StashId, HashSet<OfferIndex>>,
    /// Holds an index mapping (sell, buy) to Offer hashes to find offers for a given conversion.
    #[builder(default)]
    conversion_to_offers_idx: HashMap<ConversionIndex, HashSet<OfferIndex>>,
    // Holds a reverse index mapping Offer hash to (sell, buy) for offer invalidation.
    #[builder(default)]
    offer_to_conversion_idx: HashMap<OfferIndex, ConversionIndex>,
    #[builder(default)]
    asset_index: AssetIndex,
}

impl Store {
    pub fn new(league: League, asset_index: AssetIndex) -> Self {
        Self::builder()
            .league(league)
            .asset_index(asset_index)
            .build()
    }

    pub fn size(&self) -> usize {
        self.offers.len()
    }

    fn invalidate_stash(&mut self, stash_id: &str) {
        // Remove old index data for the given stash
        if let Some(stash_offer_indices) = self.stash_to_offers_idx.get_mut(stash_id) {
            for offer_idx in stash_offer_indices.iter() {
                // Remove offer itself
                self.offers.remove(offer_idx);
                // Clean-up conversion -> offer mapping
                if let Some(conversion_index) = self.offer_to_conversion_idx.get(offer_idx) {
                    if let Some(conversion_offer_indices) =
                        self.conversion_to_offers_idx.get_mut(conversion_index)
                    {
                        conversion_offer_indices.remove(offer_idx);
                    }
                }
                // Clean-up offer -> conversion reverse mapping
                self.offer_to_conversion_idx.remove(offer_idx);
            }
            // Clean-up but don't delete stash -> offer mapping
            stash_offer_indices.clear();
        }
    }

    fn ingest_offer(&mut self, offer: Offer) {
        let mut offer = offer;

        if let Some(mapped_item_name) = self.asset_index.get_name(&offer.buy) {
            offer.buy = mapped_item_name.to_owned();
        }

        let offer_index = offer.get_index();
        let conversion_index = Conversion::from(&offer).get_index();

        // Update index conversion -> offer
        self.conversion_to_offers_idx
            .entry(conversion_index)
            .or_default()
            .insert(offer_index);

        // Update reverse index offer -> conversion
        self.offer_to_conversion_idx
            .insert(offer_index, conversion_index);

        // Update index stash -> offer
        self.stash_to_offers_idx
            .entry(offer.stash_id.clone())
            .or_default()
            .insert(offer_index);

        // Insert new offer data
        self.offers.insert(offer_index, offer);
    }

    pub fn ingest_stash(&mut self, stash: StashRecord) -> usize {
        self.invalidate_stash(&stash.stash_id);
        tracing::trace!("Ingested {}", stash.stash_id);

        let offers: Vec<Offer> = stash.into();
        let n_offers = offers.len();

        for o in offers {
            if self.asset_index.has_item(&o.sell) {
                self.ingest_offer(o);
            }
        }

        n_offers
    }

    #[tracing::instrument(skip(self))]
    pub fn query(&self, sell: &str, buy: &str, limit: Option<usize>) -> Option<Vec<&Offer>> {
        let conversion_idx = Conversion::new(sell, buy).get_index();

        if let Some(offer_idxs) = self.conversion_to_offers_idx.get(&conversion_idx) {
            let offers = offer_idxs
                .iter()
                .filter_map(|offer_idx| self.offers.get(offer_idx))
                .take(limit.unwrap_or(50).min(100))
                .collect::<Vec<_>>();

            return Some(offers);
        }

        None
    }

    #[tracing::instrument(skip(self))]
    pub fn persist(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = bincode::serialize(&self)?;
        let file_path = format!("{}/{}.bin", STORE_BASE_PATH, self.league.to_ident());
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)?;
        file.write_all(&serialized).map_err(|e| e.into())
    }

    #[tracing::instrument]
    pub fn restore(league: &League) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}.bin", STORE_BASE_PATH, league.to_ident());
        let mut file = std::fs::OpenOptions::new().read(true).open(file_path)?;
        let store = bincode::deserialize_from(&mut file).map_err(|e| e.to_string())?;
        Ok(store)
    }
}

pub type StoreMap = HashMap<League, Arc<RwLock<Store>>>;

#[tracing::instrument]
pub async fn load_store(league: League) -> Result<Store, Box<dyn std::error::Error>> {
    let store = match Store::restore(&league) {
        Ok(store) => {
            info!("Successfully restored store from file");
            store
        }
        Err(e) => {
            error!("Error restoring store, creating new: {:?}", e);
            let mut asset_index = AssetIndex::new();
            asset_index.init().await?;
            let store = Store::new(league, asset_index);
            store.persist()?;
            store
        }
    };
    Ok(store)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        assets::AssetIndex,
        collection,
        league::League,
        source::{Item, StashRecord},
        store::Store,
    };

    #[test]
    fn test_stash_invalidation() {
        let asset_index = AssetIndex::builder()
            .long_short_idx(collection! {
                "Exalted Orb".into() => "exalted".into(),
                "Mirror of Kalandra".into() => "mirror".into()
            })
            .short_long_idx(collection! {
                "exalted".into() => "Exalted Orb".into(),
                "mirror".into() => "Mirror of Kalandra".into()
            })
            .build();

        let input = StashRecord {
            account_name: "some guy".into(),
            league: "Standard".into(),
            stash_id: "stash-id".into(),
            items: vec![
                Item {
                    id: "item-id-1".into(),
                    name: "".into(),
                    note: Some("~b/o 5 chaos".into()),
                    stack_size: Some(10),
                    type_line: "Exalted Orb".into(),
                },
                Item {
                    id: "item-id-2".into(),
                    name: "Headhunter".into(),
                    note: Some("~b/o 500 exalted".into()),
                    stack_size: Some(10),
                    type_line: "Leather Belt".into(),
                },
                Item {
                    id: "item-id-3".into(),
                    name: "".into(),
                    note: Some("~b/o 500 exalted".into()),
                    stack_size: Some(10),
                    type_line: "Mirror of Kalandra".into(),
                },
            ],
        };

        let mut store = Store::new(League::Challenge, asset_index.clone());

        store.ingest_stash(input.clone());
        store.invalidate_stash(&input.stash_id);
        store.ingest_stash(input.clone());
        store.invalidate_stash(&input.stash_id);

        let stash_to_offers_idx: HashMap<_, _> =
            collection! { "stash-id".into() => HashSet::default(), };

        let conversion_to_offers_idx: HashMap<_, _> = collection! {
            4093394393773149507 => HashSet::default(),
            11233860493351374465 => HashSet::default()
        };

        let expected = Store::builder()
            .league(League::Challenge)
            .stash_to_offers_idx(stash_to_offers_idx)
            .conversion_to_offers_idx(conversion_to_offers_idx)
            .asset_index(asset_index)
            .build();

        assert_eq!(store, expected);
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
