use prometheus_exporter::prometheus::core::{AtomicI64, AtomicU64, GenericCounter, GenericGauge};

pub trait Metrics {
    fn set_offers_ingested(&mut self, value: i64);
    fn set_stashes_ingested(&mut self, value: i64);
    fn set_store_size(&mut self, value: i64);
    fn inc_search_requests(&mut self);
}

#[derive(Clone)]
struct MetricStore {
    store_size: GenericGauge<AtomicI64>,
    offers_ingested: GenericGauge<AtomicI64>,
    stashes_ingested: GenericGauge<AtomicI64>,
    search_requests: GenericCounter<AtomicU64>,
}

impl Metrics for MetricStore {
    fn set_offers_ingested(&mut self, value: i64) {
        self.offers_ingested.set(value)
    }

    fn set_stashes_ingested(&mut self, value: i64) {
        self.stashes_ingested.set(value)
    }

    fn inc_search_requests(&mut self) {
        self.search_requests.inc()
    }

    fn set_store_size(&mut self, value: i64) {
        self.store_size.set(value)
    }
}

pub fn setup_metrics(port: u32) -> Result<impl Metrics + Clone, Box<dyn std::error::Error>> {
    let binding = format!("0.0.0.0:{}", port).parse()?;
    prometheus_exporter::start(binding)?;

    Ok(MetricStore {
        offers_ingested: prometheus_exporter::prometheus::register_int_gauge!(
            "offers_ingested",
            "The current rate of offer ingestion"
        )?,
        stashes_ingested: prometheus_exporter::prometheus::register_int_gauge!(
            "stashes_ingested",
            "The current rate of stash ingestion"
        )?,
        store_size: prometheus_exporter::prometheus::register_int_gauge!(
            "store_size",
            "The total number of ingested offers"
        )?,
        search_requests: prometheus_exporter::prometheus::register_int_counter!(
            "search_requests",
            "The number of handled search requests"
        )?,
    })
}
