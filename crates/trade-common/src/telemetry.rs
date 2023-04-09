use opentelemetry::{
    sdk::{trace::config, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn setup_telemetry(service_name: &str) -> Result<(), opentelemetry::trace::TraceError> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://otel-collector:4317"),
        )
        .with_trace_config(config().with_resource(Resource::new(vec![KeyValue::new(
            "service.name".to_string(),
            service_name.to_string(),
        )])))
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Error initialising OTLP pipeline");

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Successfully setup OTLP telemetry");

    Ok(())
}

pub fn teardown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
}
