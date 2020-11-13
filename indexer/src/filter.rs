use serde::Deserialize;

use crate::StashRecord;

pub type Filter = Box<dyn Fn(&Item) -> bool>;

pub fn filter_items_from_stash(
    mut stash_record: StashRecord,
    filters: &Vec<Filter>,
) -> (StashRecord, usize, usize) {
    let items =
        serde_json::from_value::<Vec<serde_json::Value>>(stash_record.items.clone()).unwrap();
    let n_items = items.len();

    let filtered = items
        .into_iter()
        .filter(|item| {
            serde_json::from_value::<Item>(item.clone())
                .map_or(true, |fi| filters.iter().any(|f| f(&fi)))
        })
        .collect::<Vec<_>>();

    let n_filtered = filtered.len();

    stash_record.items = serde_json::to_value(filtered).unwrap();

    (stash_record, n_items, n_items - n_filtered)
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
