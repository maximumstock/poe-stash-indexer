use std::sync::Arc;

use futures::StreamExt;
use lapin::options::BasicAckOptions;

use sqlx::{Pool, Postgres};
use tracing::info;
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

    if !ingestable_stashes.is_empty() {
        ingest(metrics, pool, league, asset_index, ingestable_stashes).await?;
    }

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
    let n_invalidated_stashes = stash_records.len();
    let stash_ids = stash_records
        .iter()
        .map(|s| format!("'{}'", s.stash_id))
        .collect::<Vec<_>>()
        .join(",");

    let query = scooby::postgres::delete_from(league.to_ident())
        .where_(format!("stash_id in ({stash_ids})"))
        .to_string();

    sqlx::query(&query)
        .bind(league.to_ident())
        .bind(&stash_ids)
        .execute(&**pool)
        .await?;

    let offers = stash_records
        .into_iter()
        .flat_map(Vec::<Offer>::from)
        .map(|o| map_offer(o, asset_index))
        .collect::<Vec<VectorizedOffer>>();

    if offers.is_empty() {
        return Ok(());
    }

    let n_ingested_offers = offers.len();
    let query = scooby::postgres::insert_into(league.to_ident())
        .columns((
            "item_id",
            "stash_id",
            "seller_account",
            "stock",
            "sell",
            "buy",
            "conversion_rate",
            "created_at",
        ))
        .values(offers)
        .to_string();

    if let Err(e) = sqlx::query(&query).execute(&**pool).await {
        tracing::error!("{}, {}", &query, e);
    }

    info!(
        "Invalidate {} stashes, Insert {} offers",
        n_invalidated_stashes, n_ingested_offers
    );

    metrics.inc_offers_ingested(n_ingested_offers as u64);
    Ok(())
}

type VectorizedOffer = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

fn map_offer(offer: Offer, asset_index: &Arc<AssetIndex>) -> VectorizedOffer {
    let sell = asset_index.get_name(&offer.sell).unwrap_or(&offer.sell);
    let buy = asset_index.get_name(&offer.buy).unwrap_or(&offer.buy);

    (
        format!("'{}'", offer.item_id),
        format!("'{}'", offer.stash_id),
        format!("'{}'", offer.seller_account),
        offer.stock.to_string(),
        format!("'{}'", escape(sell.clone())),
        format!("'{}'", escape(buy.clone())),
        format!("{}", offer.conversion_rate),
        format!("to_timestamp({})", offer.created_at),
    )
}

fn escape(s: String) -> String {
    s.replace('\'', "''")
}
