pub mod api;
pub mod store;

use crate::config::Config;

use self::{
    api::{ApiMetricStore, ApiMetrics},
    store::{StoreMetricStore, StoreMetrics},
};

pub fn setup_metrics(
    config: &Config,
) -> Result<
    (
        impl ApiMetrics + Clone + std::fmt::Debug,
        impl StoreMetrics + Clone + std::fmt::Debug,
        impl StoreMetrics + Clone + std::fmt::Debug,
    ),
    Box<dyn std::error::Error>,
> {
    let binding = format!("0.0.0.0:{}", config.metrics_port).parse()?;
    prometheus_exporter::start(binding)?;

    let api_metrics = ApiMetricStore::new();
    let store_metrics = StoreMetricStore::new(crate::league::League::Challenge);
    let store_metrics_hc = StoreMetricStore::new(crate::league::League::ChallengeHardcore);

    Ok((api_metrics, store_metrics, store_metrics_hc))
}
