use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::trace_span;
use warp::{reply::Json, Filter, Rejection, Reply};

use crate::{
    metrics::Metrics,
    store::{Offer, Store},
};

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    sell: String,
    buy: String,
}

pub async fn init<T: Into<SocketAddr> + 'static>(
    options: T,
    store: Arc<Mutex<Store>>,
    metrics: impl Metrics + Clone + Send + Sync + std::fmt::Debug + 'static,
) {
    let routes = healtcheck_endpoint()
        .or(search_endpoint(store, metrics))
        .recover(error_handler);

    warp::serve(routes).bind(options).await
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    limit: Option<usize>,
}

fn search_endpoint(
    store: Arc<Mutex<Store>>,
    metrics: impl Metrics + Clone + Send + std::fmt::Debug + 'static,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("trade"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(warp::query::<SearchQuery>())
        .and(with_store(store))
        .and(with_metrics(metrics))
        .and_then(handle_search)
}

#[tracing::instrument(skip(store, metrics))]
async fn handle_search(
    payload: RequestBody,
    query: SearchQuery,
    store: Arc<Mutex<Store>>,
    mut metrics: impl Metrics + std::fmt::Debug,
) -> Result<Json, Rejection> {
    metrics.inc_search_requests();
    let store = store.lock().await;

    if let Some(offers) = store.query(&payload.sell, &payload.buy, query.limit) {
        return Ok(warp::reply::json(&QueryResponse {
            count: offers.len(),
            offers,
        }));
    }

    Err(QueryEmptyResultError {}.into())
}

async fn error_handler(_rejection: warp::Rejection) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status(
        "error",
        warp::http::StatusCode::NOT_FOUND,
    ))
}

#[derive(Debug, Serialize)]
struct QueryEmptyResultError {}

impl warp::reject::Reject for QueryEmptyResultError {}

#[derive(Debug, Serialize)]
struct QueryResponse<'a> {
    count: usize,
    offers: Vec<&'a Offer>,
}

fn healtcheck_endpoint() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(warp::path("healthcheck"))
        .map(|| "{\"health\": \"ok\"}")
}

// #[tracing::instrument]
fn with_store(
    store: Arc<Mutex<Store>>,
) -> impl Filter<Extract = (Arc<Mutex<Store>>,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

// #[tracing::instrument]
fn with_metrics(
    metrics: impl Metrics + Clone + Send + std::fmt::Debug + 'static,
) -> impl Filter<Extract = (impl Metrics + std::fmt::Debug,), Error = Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}
