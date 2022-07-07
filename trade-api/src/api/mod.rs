mod routes;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Extension, Router,
};

use tower::ServiceBuilder;

use crate::{metrics::api::ApiMetrics, store::Store};

use self::routes::search::handle_search;

pub async fn init<T: Into<SocketAddr> + 'static>(
    options: T,
    metrics: impl ApiMetrics + Send + Sync,
    store: Arc<Store>,
) {
    let app = Router::new()
        .route("/healthcheck", get(health_handler))
        .route("/trade", post(handle_search))
        .layer(
            ServiceBuilder::new().layer(Extension(store)), // .layer(Extension(metrics)),
        );

    axum::Server::bind(&options.into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument()]
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Ok")
}

#[tracing::instrument()]
async fn error_handler(error: BoxError) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
