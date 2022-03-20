use std::sync::Arc;

use futures::StreamExt;
use lapin::options::BasicAckOptions;
use tokio::sync::{oneshot::Receiver, Mutex};
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
    mut shutdown_rx: Receiver<()>,
    store: Arc<Mutex<Store>>,
    mut metrics: impl Metrics + std::fmt::Debug,
) -> Result<(), Box<dyn std::error::Error>> {
    // todo: we are trapped here when stopping signal comes
    let mut consumer = retry_setup_consumer(config).await?;

    while let Some(delivery) = consumer.next().await {
        if delivery.is_err() {
            // todo: we are trapped here when stopping signal comes
            consumer = retry_setup_consumer(config).await?;
        }

        let delivery = delivery?;
        consume(&delivery, &mut metrics, &store).await?;
        delivery.ack(BasicAckOptions::default()).await?;

        if let Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) =
            shutdown_rx.try_recv()
        {
            break;
        }
    }

    info!("Stopping RabbitMQ consumer");

    Ok(())
}

#[tracing::instrument(skip(store, metrics, delivery))]
async fn consume(
    delivery: &lapin::message::Delivery,
    metrics: &mut (impl Metrics + std::fmt::Debug),
    store: &Arc<Mutex<Store>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = tracing::info_span!("deserialization")
        .in_scope(|| serde_json::from_slice::<Vec<StashRecord>>(&delivery.data))?;
    metrics.set_stashes_ingested(stash_records.len() as i64);

    let mut store = store.lock().await;
    let n_ingested_offers = tracing::info_span!("ingestion")
        .in_scope(|| async {
            stash_records
                .into_iter()
                .map(|s| store.ingest_stash(s))
                .sum::<usize>()
        })
        .await;

    metrics.set_offers_ingested(n_ingested_offers as i64);
    info!("Store has {:#?} offers", store.size());
    metrics.set_store_size(store.size() as i64);
    Ok(())
}
