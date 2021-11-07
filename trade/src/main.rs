/// TODO
/// - RabbitMQ client that produces a stream of `StashRecord`s
/// - a module to maintain `StashRecord`s as offers /w indices to answer:
///   - What offers are there for selling X for Y?
///   - What offers can we delete if a new stash is updated
///   - turning `StashRecord` into a set of Offers
/// - a web API that mimics pathofexile.com/trade API
/// - will need state snapshots + restoration down the road

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Clone)]
struct StashRecord {
    stash_id: String,
    league: String,
    account_name: String,
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
struct Item {
    item_id: String,
    type_line: String,
    note: Option<String>,
    stack_size: u32,
}

mod store {
    use std::{
        collections::{hash_map::DefaultHasher, HashMap, HashSet},
        fmt::Debug,
        hash::{Hash, Hasher},
    };
    use typed_builder::TypedBuilder;

    use crate::StashRecord;

    type StashId = String;
    type ItemId = String;
    type OfferIndex = u64;

    #[derive(Debug)]
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
    }

    impl Default for Offer {
        fn default() -> Self {
            Self {
                item_id: "herp_derp".into(),
                stash_id: "my_stash".into(),
                sell: "chaos".into(),
                buy: "exa".into(),
                conversion_rate: (1f32 / 100f32),
                seller_account: "some guy".into(),
                stock: 1000,
            }
        }
    }

    impl Offer {
        pub fn new() -> Self {
            Self::default()
        }

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

            stash
                .items
                .into_iter()
                .map(|item| Offer {
                    stock: item.stack_size,
                    buy: item.type_line,
                    // TODO parse item (from abbreviation to full name) & price
                    conversion_rate: 0f32,
                    sell: item.note.unwrap_or_default(),
                    item_id: item.item_id,
                    seller_account: account_name.clone(),
                    stash_id: stash_id.clone(),
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

    #[derive(Debug, TypedBuilder, Eq, PartialEq)]
    struct Store {
        league: String,
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
    }

    impl Store {
        pub fn new(league: impl ToString) -> Self {
            Self::builder().league(league.to_string()).build()
        }

        pub fn invalidate_stash(&mut self, stash_id: &str) {
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

        pub fn ingest_offer(&mut self, offer: Offer) {
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

        pub fn ingest_stash(&mut self, stash: StashRecord) {
            self.invalidate_stash(&stash.stash_id);

            let offers: Vec<Offer> = stash.into();
            for o in offers {
                self.ingest_offer(o);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::collections::{HashMap, HashSet};

        use crate::{collection, store::Store, Item, StashRecord};

        #[test]
        fn test_stash_invalidation() {
            let input = StashRecord {
                account_name: "some guy".into(),
                league: "Standard".into(),
                stash_id: "stash-id".into(),
                items: vec![Item {
                    item_id: "item-id-1".into(),
                    note: Some("~5 chaos".into()),
                    stack_size: 10,
                    type_line: "Exalted Orb".into(),
                }],
            };

            let mut store = Store::new("Standard");

            store.ingest_stash(input.clone());
            store.invalidate_stash(&input.stash_id);
            store.ingest_stash(input.clone());
            store.invalidate_stash(&input.stash_id);

            let stash_to_offers_idx2: HashMap<_, _> =
                collection! { "stash-id".into() => HashSet::default(), };

            let conversion_to_offers_idx: HashMap<_, _> =
                collection! { 11579197031891085763 => HashSet::default() };

            let expected = Store::builder()
                .league("Standard".into())
                .stash_to_offers_idx(stash_to_offers_idx2)
                .conversion_to_offers_idx(conversion_to_offers_idx)
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
}
