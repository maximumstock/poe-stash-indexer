mod routes;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};

use tower::ServiceBuilder;

use crate::{metrics::api::ApiMetrics, store::Store};

use self::routes::search::handle_search;

pub async fn init<T: Into<SocketAddr> + 'static, M: ApiMetrics>(
    options: T,
    metrics: M,
    store: Arc<Store>,
) {
    let app = Router::new()
        .route("/healthcheck", get(health_handler))
        .route("/trade", post(handle_search::<M>))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(store))
                .layer(Extension(metrics)),
        );

    let _ = axum::Server::bind(&options.into())
        .serve(app.into_make_service())
        .await;
}

#[tracing::instrument()]
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Ok")
}
