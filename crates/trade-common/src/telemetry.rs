use opentelemetry::{
    sdk::{trace::config, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
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
        .with_trace_config(config().with_resource(Resource::new(vec![
            KeyValue::new("service.name".to_string(), service_name.to_string()),
            KeyValue::new(
                "deployment.environment",
                std::env::var("ENV").unwrap_or("development".into()),
            ),
        ])))
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

pub fn generate_http_client() -> reqwest_middleware::ClientWithMiddleware {
    let reqwest_client = reqwest::Client::builder().build().unwrap();

    ClientBuilder::new(reqwest_client)
        // Insert the tracing middleware
        .with(TracingMiddleware::default())
        .build()
}
