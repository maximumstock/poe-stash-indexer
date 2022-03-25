use std::sync::Arc;

use futures::StreamExt;
use lapin::options::BasicAckOptions;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

use crate::{
    config::Config,
    metrics::Metrics,
    source::{retry_setup_consumer, ExampleStream, StashRecord},
    store::Store,
};

#[allow(dead_code)]
async fn setup_local_consumer(store: Arc<Mutex<Store>>) {
    let example_stream = ExampleStream::new("./data.json");

    for stash_record in example_stream {
        let mut store = store.lock().await;
        store.ingest_stash(stash_record);
        info!("Store has {:#?} offers", store.size());
        drop(store);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub async fn setup_rabbitmq_consumer(
    config: &Config,
    store: Arc<RwLock<Store>>,
    mut metrics: impl Metrics + std::fmt::Debug,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initial connection should be retried until it works
    let mut consumer = retry_setup_consumer(config).await;

    loop {
        if let Some(delivery) = consumer.next().await {
            match delivery {
                Err(_) => {
                    // This takes care of reconnection if the connection drops after the initial connect
                    consumer = retry_setup_consumer(config).await;
                }
                Ok(delivery) => {
                    consume(&delivery, &mut metrics, &store).await?;
                    delivery.ack(BasicAckOptions::default()).await?;
                }
            }
        }
    }
}

#[tracing::instrument(skip(store, metrics, delivery))]
async fn consume(
    delivery: &lapin::message::Delivery,
    metrics: &mut (impl Metrics + std::fmt::Debug),
    store: &Arc<RwLock<Store>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = tracing::info_span!("deserialization")
        .in_scope(|| serde_json::from_slice::<Vec<StashRecord>>(&delivery.data))?;
    metrics.set_stashes_ingested(stash_records.len() as i64);

    let mut store = store.write().await;
    let n_ingested_offers = tracing::info_span!("ingestion").in_scope(|| {
        stash_records
            .into_iter()
            .map(|s| store.ingest_stash(s))
            .sum::<usize>()
    });

    metrics.set_offers_ingested(n_ingested_offers as i64);
    info!("Store has {:#?} offers", store.size());
    metrics.set_store_size(store.size() as i64);
    Ok(())
}
