use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};

pub trait ApiMetrics {
    fn inc_search_requests(&mut self);
}

#[derive(Clone, Debug)]
pub struct ApiMetricStore {
    search_requests: GenericCounter<AtomicU64>,
}

impl ApiMetrics for ApiMetricStore {
    fn inc_search_requests(&mut self) {
        self.search_requests.inc()
    }
}

impl ApiMetricStore {
    pub fn new() -> Self {
        ApiMetricStore {
            search_requests: prometheus_exporter::prometheus::register_int_counter!(
                "search_requests",
                "The number of handled search requests"
            )
            .unwrap(),
        }
    }
}
