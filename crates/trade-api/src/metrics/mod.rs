pub mod api;

use crate::config::Config;

use self::api::{ApiMetricStore, ApiMetrics};

pub fn setup_metrics(config: &Config) -> Result<impl ApiMetrics, Box<dyn std::error::Error>> {
    let binding = format!("0.0.0.0:{}", config.metrics_port).parse()?;
    prometheus_exporter::start(binding)?;

    let api_metrics = ApiMetricStore::new();

    Ok(api_metrics)
}
