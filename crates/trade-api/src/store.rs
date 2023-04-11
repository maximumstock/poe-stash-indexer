use serde::Serialize;
use sqlx::query_builder::QueryBuilder;
use sqlx::{FromRow, Pool, Postgres};
use std::fmt::Debug;
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
    /// The quantity of [buy] units the seller gets for 1 unit of [sell]
    conversion_rate: f32,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, TypedBuilder)]
pub struct Store {
    #[builder(default)]
    asset_index: AssetIndex,
    pool: Pool<Postgres>,
}

impl Store {
    pub fn new(asset_index: AssetIndex, pool: Pool<Postgres>) -> Self {
        Self::builder().asset_index(asset_index).pool(pool).build()
    }

    #[tracing::instrument(skip(self))]
    pub async fn query(
        &self,
        league: League,
        mut query: StoreQuery,
    ) -> Result<Vec<Offer>, Box<dyn std::error::Error>> {
        if let Some(ref buy) = query.buy {
            query.buy = self.asset_index.get_name(buy).cloned().or(query.buy);
        }

        if let Some(ref sell) = query.sell {
            query.sell = self.asset_index.get_name(sell).cloned().or(query.sell);
        }

        let mut builder = QueryBuilder::<Postgres>::new(format!(
            "SELECT * FROM {} WHERE 1=1 ",
            league.to_ident()
        ));

        if let Some(sell) = query.sell {
            builder.push("AND sell = ").push_bind(sell);
        }

        if let Some(buy) = query.buy {
            builder.push("AND buy = ").push_bind(buy);
        }

        if let Some(seller_account) = query.seller_account {
            builder
                .push("AND seller_account = ")
                .push_bind(seller_account);
        }

        if let Some(stash_id) = query.stash_id {
            builder.push("AND stash_id = ").push_bind(stash_id);
        }

        builder
            .push("ORDER BY created_at DESC ")
            .push("LIMIT ")
            .push_bind(query.limit.map(|l| (l as i32).min(200)).or(Some(50)));

        let offers = builder
            .build()
            .try_map(|row| Offer::from_row(&row))
            .fetch_all(&self.pool)
            .await?;

        Ok(offers)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StoreQuery {
    pub(crate) sell: Option<String>,
    pub(crate) buy: Option<String>,
    pub(crate) seller_account: Option<String>,
    pub(crate) stash_id: Option<String>,
    pub(crate) limit: Option<u32>,
}
