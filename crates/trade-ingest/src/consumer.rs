use std::sync::Arc;

use chrono::DateTime;
use futures::StreamExt;
use lapin::options::BasicAckOptions;

use sqlx::{Execute, Pool, Postgres, QueryBuilder};
use tracing::{info, trace};
use trade_common::{assets::AssetIndex, league::League};

use crate::{
    config::Config,
    metrics::store::StoreMetrics,
    source::{retry_setup_consumer, StashRecord},
    store::Offer,
};

pub async fn setup_rabbitmq_consumer(
    config: &Config,
    pool: Arc<Pool<Postgres>>,
    mut metrics: impl StoreMetrics,
    asset_index: Arc<AssetIndex>,
    league: League,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initial connection should be retried until it works
    let mut consumer = retry_setup_consumer(config, &league).await;

    loop {
        if let Some(delivery) = consumer.next().await {
            match delivery {
                Err(_) => {
                    // This takes care of reconnection if the connection drops after the initial connect
                    consumer = retry_setup_consumer(config, &league).await;
                }
                Ok(delivery) => {
                    consume(&delivery, &pool, &mut metrics, &asset_index, &league).await?;
                    delivery.ack(BasicAckOptions::default()).await?;
                }
            }
        }
    }
}

#[tracing::instrument(skip(metrics, pool, asset_index, delivery))]
async fn consume(
    delivery: &lapin::message::Delivery,
    pool: &Arc<Pool<Postgres>>,
    metrics: &mut impl StoreMetrics,
    asset_index: &Arc<AssetIndex>,
    league: &League,
) -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = tracing::info_span!("deserialization")
        .in_scope(|| serde_json::from_slice::<Vec<StashRecord>>(&delivery.data))?;

    metrics.inc_stashes_ingested(stash_records.len() as u64);

    let ingestable_stashes = stash_records
        .into_iter()
        .filter(|s| s.league.eq(league.to_str()))
        .collect::<Vec<_>>();

    ingest(metrics, pool, league, asset_index, ingestable_stashes).await?;

    Ok(())
}

#[tracing::instrument(skip(metrics, pool, asset_index, stash_records))]
async fn ingest(
    metrics: &mut impl StoreMetrics,
    pool: &Arc<Pool<Postgres>>,
    league: &League,
    asset_index: &Arc<AssetIndex>,
    stash_records: Vec<StashRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    let n_invalidated_stashes = stash_records.len() as u64;

    let stash_ids = stash_records
        .iter()
        .map(|s| format!("'{}'", s.stash_id))
        .collect::<Vec<_>>()
        .join(",");

    sqlx::query(&format!(
        "DELETE FROM {} WHERE stash_id in ($1)",
        league.to_ident()
    ))
    .bind(&stash_ids)
    .execute(&**pool)
    .await?;

    trace!(n_invalidated_stashes = stash_records.len());

    let stash_offers = stash_records
        .into_iter()
        .flat_map(Vec::<Offer>::from)
        .collect::<Vec<_>>();

    let n_ingested_offers = stash_offers.len() as u64;
    trace!(n_ingested_offers);

    if stash_offers.is_empty() {
        return Ok(());
    }

    let mut query_builder = QueryBuilder::<Postgres>::new(format!(
        "INSERT INTO {} ({}) ",
        league.to_ident(),
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

    if let Err(e) = ingest_query.execute(&**pool).await {
        tracing::error!("{}, {}", &ingest_query_str, e);
        return Err(e.into());
    }

    info!(
        "Invalidate {} stashes, Insert {} offers",
        n_invalidated_stashes, n_ingested_offers
    );

    metrics.inc_offers_ingested(n_ingested_offers);

    Ok(())
}
