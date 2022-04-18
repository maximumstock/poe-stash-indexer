mod middleware;
mod routes;

use std::{net::SocketAddr, sync::Arc};

use serde::Serialize;

use warp::{Filter, Rejection, Reply};

use crate::{
    metrics::api::ApiMetrics,
    store::{Offer, Store},
};

use self::routes::{healthcheck::healtcheck_endpoint, search::search};

pub async fn init<T: Into<SocketAddr> + 'static>(
    options: T,
    metrics: impl ApiMetrics + Send + Sync + 'static,
    store: Arc<Store>,
) {
    let routes = healtcheck_endpoint()
        .or(search(metrics, store))
        .with(warp::trace::request())
        .recover(error_handler);

    warp::serve(routes).bind(options).await
}

#[tracing::instrument(skip(_rejection))]
async fn error_handler(_rejection: warp::Rejection) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status(
        "error",
        warp::http::StatusCode::NOT_FOUND,
    ))
}

#[derive(Debug, Serialize)]
struct QueryEmptyResultError {}

impl QueryEmptyResultError {
    pub fn new() -> Self {
        Self {}
    }
}

impl warp::reject::Reject for QueryEmptyResultError {}

#[derive(Debug, Serialize)]
struct QueryResponse {
    count: usize,
    offers: Vec<Offer>,
}

impl QueryResponse {
    fn new(offers: Vec<Offer>) -> Self {
        Self {
            count: offers.len(),
            offers,
        }
    }
}
