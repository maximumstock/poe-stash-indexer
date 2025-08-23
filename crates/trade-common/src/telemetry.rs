use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use reqwest_leaky_bucket::leaky_bucket::RateLimiter;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn setup_telemetry(service_name: &str) -> Result<(), opentelemetry::trace::TraceError> {
    if let Ok(otel_collector) = std::env::var("OTEL_COLLECTOR") {
        println!("Connecting to OTEL_COLLECTOR {}", otel_collector);
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&otel_collector),
            )
            .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
                Resource::new(vec![
                    opentelemetry::KeyValue::new(
                        "service.name".to_string(),
                        service_name.to_string(),
                    ),
                    opentelemetry::KeyValue::new(
                        "deployment.environment",
                        std::env::var("ENV").unwrap_or("development".into()),
                    ),
                ]),
            ))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Error initialising OTLP pipeline");

        Registry::default()
            .with(EnvFilter::from_default_env())
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer())
            .init();

        info!("Setup tracing with OTLP ({})", otel_collector);
    } else {
        Registry::default()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        info!("Setup tracing without OTLP");
    }

    info!("Successfully setup telemetry");

    Ok(())
}

pub fn teardown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// Sets up a reusable tracing-first reqwest client with optional rate limiting
pub fn generate_http_client(
    rate_limiter: Option<RateLimiter>,
) -> reqwest_middleware::ClientWithMiddleware {
    let reqwest_client = reqwest::Client::builder().build().unwrap();

    if let Some(limiter) = rate_limiter {
        ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .with(reqwest_leaky_bucket::rate_limit_all(limiter))
            .build()
    } else {
        ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .build()
    }
}
