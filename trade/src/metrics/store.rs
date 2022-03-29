use prometheus_exporter::prometheus::core::{AtomicI64, AtomicU64, GenericCounter, GenericGauge};

use crate::league::League;

pub trait StoreMetrics {
    fn inc_offers_ingested(&mut self, value: u64);
    fn inc_stashes_ingested(&mut self, value: u64);
    fn set_store_size(&mut self, value: i64);
}

#[derive(Clone, Debug)]
pub struct StoreMetricStore {
    store_size: GenericGauge<AtomicI64>,
    offers_ingested: GenericCounter<AtomicU64>,
    stashes_ingested: GenericCounter<AtomicU64>,
}

impl StoreMetrics for StoreMetricStore {
    fn inc_offers_ingested(&mut self, value: u64) {
        self.offers_ingested.inc_by(value)
    }

    fn inc_stashes_ingested(&mut self, value: u64) {
        self.stashes_ingested.inc_by(value)
    }

    fn set_store_size(&mut self, value: i64) {
        self.store_size.set(value)
    }
}

impl StoreMetricStore {
    pub fn new(league: League) -> Self {
        StoreMetricStore {
            offers_ingested: prometheus_exporter::prometheus::register_int_counter!(
                format!("offers_ingested_{}", league.to_ident()),
                "The current rate of offer ingestion"
            )
            .unwrap(),
            stashes_ingested: prometheus_exporter::prometheus::register_int_counter!(
                format!("stashes_ingested_{}", league.to_ident()),
                "The current rate of stash ingestion"
            )
            .unwrap(),
            store_size: prometheus_exporter::prometheus::register_int_gauge!(
                format!("store_size_{}", league.to_ident()),
                "The total number of ingested offers"
            )
            .unwrap(),
        }
    }
}
