use serde::Serialize;
use sqlx::{FromRow, Pool, Postgres};
use std::{fmt::Debug, sync::Arc};
use trade_common::{assets::AssetIndex, league::League};

use typed_builder::TypedBuilder;

type StashId = String;
type ItemId = String;

#[derive(Debug, Serialize, FromRow)]
/// Describes an offer from the view of the seller.
pub struct Offer {
    item_id: ItemId,
    stash_id: StashId,
    /// Item that is sold from the point of view of the seller.
    sell: String,
    /// Item that the seller receives.
    buy: String,
    seller_account: String,
    stock: i32,
    conversion_rate: f32,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, TypedBuilder)]
pub struct Store {
    #[builder(default)]
    asset_index: AssetIndex,
    pool: Arc<Pool<Postgres>>,
}

impl Store {
    pub fn new(asset_index: AssetIndex, pool: Arc<Pool<Postgres>>) -> Self {
        Self::builder().asset_index(asset_index).pool(pool).build()
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_offers(
        &self,
        league: League,
        sell: String,
        buy: String,
        limit: Option<u32>,
    ) -> Result<Vec<Offer>, Box<dyn std::error::Error>> {
        let buy = self.asset_index.get_name(&buy).unwrap_or(&buy);
        let sell = self.asset_index.get_name(&sell).unwrap_or(&sell);
        let limit = limit.unwrap_or(50).min(200);

        let query = format!(
            "SELECT * FROM {} WHERE buy = $1 and sell = $2 ORDER BY created_at DESC LIMIT $3",
            league.to_ident()
        );
        let offers = sqlx::query_as(&query)
            .bind(buy)
            .bind(sell)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await?;

        Ok(offers)
    }
}
