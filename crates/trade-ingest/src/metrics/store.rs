use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use trade_common::league::League;

pub trait StoreMetrics: Clone + std::fmt::Debug {
    fn inc_offers_ingested(&mut self, value: u64);
    fn inc_stashes_ingested(&mut self, value: u64);
}

#[derive(Clone, Debug)]
pub struct StoreMetricStore {
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
        }
    }
}
