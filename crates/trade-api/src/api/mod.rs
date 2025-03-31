mod routes;

use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Extension, Router,
};

use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info, Level};

use crate::{metrics::api::ApiMetrics, store::Store};

use self::routes::search::handle_search;

pub async fn init<T, M>(options: T, metrics: M, store: Arc<Store>)
where
    T: Into<SocketAddr> + 'static + Debug,
    M: ApiMetrics,
{
    let app = Router::new()
        .route("/healthcheck", get(health_handler))
        .route("/trade", post(handle_search::<M>))
        .layer(trace_layer())
        .layer(
            ServiceBuilder::new()
                .layer(Extension(store))
                .layer(Extension(metrics)),
        );

    info!("Starting API: {options:?}");

    let listener = tokio::net::TcpListener::bind(options.into()).await.unwrap();

    let _ = axum::serve(listener, app.into_make_service()).await;
}

fn trace_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .include_headers(true)
                .latency_unit(LatencyUnit::Micros),
        )
}

#[tracing::instrument()]
async fn health_handler() -> Response {
    (StatusCode::OK, "Ok").into_response()
}
