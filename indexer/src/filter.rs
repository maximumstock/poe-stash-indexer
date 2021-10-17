use serde::Deserialize;

use crate::{config::config::Configuration, stash_record::StashRecord};

pub enum FilterResult {
    Filter { n_total: usize, n_retained: usize },
    Block { reason: String },
    Pass,
}

pub fn filter_stash_record(stash_record: &mut StashRecord, config: &Configuration) -> FilterResult {
    // League filtering
    let league = stash_record.league.clone();
    let allowed_leagues = config
        .user_config
        .filter
        .leagues
        .clone()
        .unwrap_or_default();

    if league.is_some()
        && !allowed_leagues.is_empty()
        && !allowed_leagues.contains(&league.clone().unwrap())
    {
        return FilterResult::Block {
            reason: format!("League \"{}\" blocked", league.unwrap()),
        };
    }

    // Item filtering
    let allowed_item_categories = config
        .user_config
        .filter
        .item_categories
        .clone()
        .unwrap_or_default();

    if !allowed_item_categories.is_empty() {
        let items =
            serde_json::from_value::<Vec<serde_json::Value>>(stash_record.items.clone()).unwrap();

        let n_total = items.len();

        let filtered = items
            .into_iter()
            .filter(|item| {
                serde_json::from_value::<Item>(item.clone()).map_or(true, |i| {
                    allowed_item_categories.contains(&i.extended.category)
                })
            })
            .collect::<Vec<_>>();

        let n_filtered = filtered.len();

        if n_filtered == 0 {
            return FilterResult::Block {
                reason: "Item category filter removed all items".to_string(),
            };
        }

        stash_record.items = serde_json::to_value(filtered).unwrap();

        return FilterResult::Filter {
            n_total,
            n_retained: n_filtered,
        };
    }

    FilterResult::Pass
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub type_line: String,
    pub extended: ItemExtendedProp,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemExtendedProp {
    pub category: String,
    pub base_type: String,
}

// @todo add tests
#[cfg(test)]
mod test {
    #[test]
    fn test_league_filter() {}

    #[test]
    fn test_item_category_filter() {}
}
