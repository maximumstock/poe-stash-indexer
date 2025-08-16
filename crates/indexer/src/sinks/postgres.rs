use async_trait::async_trait;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Execute, Pool, Postgres, QueryBuilder};
use stash_api::common::stash::Stash;
use tracing::{debug, trace};
use trade_common::{assets::AssetIndex, league::League, note_parser::PriceParser};

use crate::config::ensure_string_from_env;

use super::sink::Sink;

#[derive(Debug)]
pub struct PostgresSink {
    pool: Pool<Postgres>,
    asset_index: AssetIndex,
}

impl PostgresSink {
    #[tracing::instrument]
    pub async fn connect(
        config: &PostgresConfig,
        asset_index: AssetIndex,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.connection_url)
            .await?;

        // Migrate after connecting
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool, asset_index })
    }
}

#[async_trait]
impl Sink for PostgresSink {
    #[tracing::instrument(skip(self, payload), name = "sink-handle-postgres")]
    async fn handle(&mut self, payload: &[Stash]) -> Result<usize, Box<dyn std::error::Error>> {
        // todo: sink-specific metrics
        // metrics.inc_stashes_ingested(stash_records.len() as u64);

        // todo: group stash records by league and call ingest
        // replace non-ascii by `_` and run migration for newly discovered leagues
        let league = League::new("challenge".to_string());
        self.ingest(&self.pool, &league, &self.asset_index, payload)
            .await?;

        Ok(payload.len())
    }

    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl PostgresSink {
    #[tracing::instrument(skip(self, pool, asset_index, stash_records))]
    async fn ingest(
        // metrics: &mut impl StoreMetrics,
        &self,
        pool: &Pool<Postgres>,
        league: &League,
        asset_index: &AssetIndex,
        stash_records: &[Stash],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let n_invalidated_stashes = stash_records.len() as u64;

        let stash_ids = stash_records
            .iter()
            .map(|s| format!("'{}'", s.id))
            .collect::<Vec<_>>()
            .join(",");

        sqlx::query(&format!(
            "DELETE FROM {} WHERE stash_id in ($1)",
            league.as_ref()
        ))
        .bind(&stash_ids)
        .execute(pool)
        .await?;

        trace!(n_invalidated_stashes = stash_records.len());

        let stash_offers = stash_records
            .iter()
            .flat_map(map_stash_to_offers)
            .collect::<Vec<_>>();

        let n_ingested_offers = stash_offers.len() as u64;
        trace!(n_ingested_offers);

        if stash_offers.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::<Postgres>::new(format!(
            "INSERT INTO {} ({}) ",
            league.as_ref(),
            [
                "item_id",
                "stash_id",
                "seller_account",
                "stock",
                "sell",
                "buy",
                "conversion_rate",
                "created_at"
            ]
            .join(", ")
        ));

        query_builder.push_values(stash_offers.iter(), |mut query, o| {
            query.push_bind(&o.item_id);
            query.push_bind(&o.stash_id);
            query.push_bind(&o.seller_account);
            query.push_bind(o.stock as i64);
            query.push_bind(asset_index.get_name(&o.sell).unwrap_or(&o.sell).clone());
            query.push_bind(asset_index.get_name(&o.buy).unwrap_or(&o.buy).clone());
            query.push_bind(o.conversion_rate);
            query.push_bind(DateTime::from_timestamp_millis(o.created_at as i64));
        });

        let ingest_query = query_builder.build();
        let ingest_query_str = ingest_query.sql().to_string();

        if let Err(e) = ingest_query.execute(pool).await {
            tracing::error!("{}, {}", &ingest_query_str, e);
            return Err(e.into());
        }

        debug!(
            "Invalidate {} stashes, Insert {} offers",
            n_invalidated_stashes, n_ingested_offers
        );

        // metrics.inc_offers_ingested(n_ingested_offers);

        Ok(())
    }
}

type StashId = String;
type ItemId = String;

#[derive(Debug, Serialize, Deserialize)]
/// Describes an offer from the view of the seller.
pub struct Offer {
    pub(crate) item_id: Option<ItemId>,
    pub(crate) stash_id: StashId,
    /// Item that is sold from the point of view of the seller.
    pub(crate) sell: String,
    /// Item that the seller receives.
    pub(crate) buy: String,
    pub(crate) seller_account: String,
    pub(crate) stock: u16,
    pub(crate) conversion_rate: f32,
    pub(crate) created_at: u64,
}

fn map_stash_to_offers(stash: &Stash) -> Vec<Offer> {
    let account_name = stash.account_name.clone();
    let stash_id = stash.id.clone();
    let price_parser = PriceParser::new();

    stash
        .items
        .iter()
        .filter(|item| item.note.is_some())
        .filter_map(|item| {
            if let Some(price) = price_parser.parse_price(&item.note.clone().unwrap()) {
                let sold_item_name = match item.name.as_str() {
                    "" => item.type_line.clone(),
                    _ => item.name.clone(),
                };

                Some(Offer {
                    stock: item.stack_size.unwrap_or(1_u16),
                    sell: sold_item_name,
                    conversion_rate: price.ratio,
                    buy: price.item.to_owned(),
                    item_id: item.id.clone(),
                    seller_account: account_name.clone().unwrap_or("".to_owned()),
                    stash_id: stash_id.clone(),
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Failed to create timestamp")
                        .as_millis() as u64,
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PostgresConfig {
    pub connection_url: String,
}

impl PostgresConfig {
    #[allow(dead_code)]
    pub fn from_env() -> Result<Option<PostgresConfig>, std::env::VarError> {
        if let Ok(string) = std::env::var("POSTGRES_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let connection_url = ensure_string_from_env("POSTGRES_URL");

            Ok(Some(PostgresConfig { connection_url }))
        } else {
            Ok(None)
        }
    }
}
