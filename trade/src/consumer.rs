use std::sync::Arc;

use futures::StreamExt;
use lapin::options::BasicAckOptions;
use tracing::info;

use crate::{
    config::Config,
    league::League,
    metrics::store::StoreMetrics,
    source::{retry_setup_consumer, StashRecord},
    store::StoreMap,
};

pub async fn setup_rabbitmq_consumer(
    config: &Config,
    store_map: Arc<StoreMap>,
    mut metrics: impl StoreMetrics + std::fmt::Debug,
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
                    consume(&delivery, &mut metrics, &store_map, &league).await?;
                    delivery.ack(BasicAckOptions::default()).await?;
                }
            }
        }
    }
}

#[tracing::instrument(skip(store_map, metrics, delivery))]
async fn consume(
    delivery: &lapin::message::Delivery,
    metrics: &mut (impl StoreMetrics + std::fmt::Debug),
    store_map: &StoreMap,
    league: &League,
) -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = tracing::info_span!("deserialization")
        .in_scope(|| serde_json::from_slice::<Vec<StashRecord>>(&delivery.data))?;

    metrics.inc_stashes_ingested(stash_records.len() as u64);

    let stash_records = stash_records
        .into_iter()
        .filter(|s| s.league.eq(league.to_str()))
        .collect::<Vec<_>>();

    if !stash_records.is_empty() {
        ingest(store_map, metrics, league, stash_records).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(store_map, metrics, stash_records))]
async fn ingest(
    store_map: &StoreMap,
    metrics: &mut (impl StoreMetrics + std::fmt::Debug),
    league: &League,
    stash_records: Vec<StashRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(store) = store_map.get(league) {
        let mut store = store.write().await;
        let n_ingested_offers = stash_records
            .into_iter()
            .map(|s| store.ingest_stash(s))
            .sum::<usize>();

        metrics.inc_offers_ingested(n_ingested_offers as u64);
        info!("{} store has {:#?} offers", league.to_str(), store.size());
        metrics.set_store_size(store.size() as i64);
    }
    Ok(())
}
