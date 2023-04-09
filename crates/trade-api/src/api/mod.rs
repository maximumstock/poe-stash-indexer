mod routes;

use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};

use tower::ServiceBuilder;
use tracing::info;

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
        .layer(
            ServiceBuilder::new()
                .layer(Extension(store))
                .layer(Extension(metrics)),
        );

    info!("Starting API: {options:?}");

    let _ = axum::Server::bind(&options.into())
        .serve(app.into_make_service())
        .await;
}

#[tracing::instrument()]
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Ok")
}
