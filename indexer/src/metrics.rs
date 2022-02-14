use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};

pub struct Metrics {
    pub chunks_processed: GenericCounter<AtomicU64>,
    pub stashes_processed: GenericCounter<AtomicU64>,
}

pub fn setup_metrics(port: u32) -> Result<Metrics, Box<dyn std::error::Error>> {
    let binding = format!("0.0.0.0:{}", port).parse()?;
    prometheus_exporter::start(binding)?;

    let chunks_processed =
        prometheus_exporter::prometheus::register_int_counter!("chunks_processed", "help")?;

    let stashes_processed =
        prometheus_exporter::prometheus::register_int_counter!("stashes_processed", "help")?;

    Ok(Metrics {
        chunks_processed,
        stashes_processed,
    })
}
