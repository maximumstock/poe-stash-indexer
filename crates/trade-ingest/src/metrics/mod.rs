pub mod store;

use trade_common::league::League;

use crate::config::Config;

use self::store::{StoreMetricStore, StoreMetrics};

pub fn setup_metrics(
    config: &Config,
    challenge_league: &League,
    challenge_league_hc: &League,
) -> Result<(impl StoreMetrics, impl StoreMetrics), Box<dyn std::error::Error>> {
    let binding = format!("0.0.0.0:{}", config.metrics_port).parse()?;
    prometheus_exporter::start(binding)?;

    let store_metrics = StoreMetricStore::new(challenge_league);
    let store_metrics_hc = StoreMetricStore::new(challenge_league_hc);

    Ok((store_metrics, store_metrics_hc))
}
