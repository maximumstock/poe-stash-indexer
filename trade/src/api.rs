use std::{convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use warp::{reply::Json, Filter, Rejection, Reply};

use crate::{
    league::League,
    metrics::api::ApiMetrics,
    store::{Offer, StoreMap},
};

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    sell: String,
    buy: String,
}

pub async fn init<T: Into<SocketAddr> + 'static>(
    options: T,
    store_map: Arc<StoreMap>,
    metrics: impl ApiMetrics + Clone + Send + Sync + std::fmt::Debug + 'static,
) {
    let routes = healtcheck_endpoint()
        .or(search_endpoint(store_map, metrics))
        .with(warp::trace::request())
        .recover(error_handler);

    warp::serve(routes).bind(options).await
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    limit: Option<usize>,
    league: Option<String>,
}

fn search_endpoint(
    store_map: Arc<StoreMap>,
    metrics: impl ApiMetrics + Clone + Send + std::fmt::Debug + 'static,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("trade"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(warp::query::<SearchQuery>())
        .and(with_store(store_map))
        .and(with_metrics(metrics))
        .and_then(handle_search)
}

#[tracing::instrument(skip(store_map, metrics))]
async fn handle_search(
    payload: RequestBody,
    query: SearchQuery,
    store_map: Arc<StoreMap>,
    mut metrics: impl ApiMetrics + std::fmt::Debug,
) -> Result<Json, Rejection> {
    metrics.inc_search_requests();
    let league = match query.league {
        Some(s) => League::from_str(&s).unwrap_or_default(),
        None => League::default(),
    };

    match store_map.get(&league) {
        Some(store) => {
            let store = store.read().await;

            if let Some(offers) = store.query(&payload.sell, &payload.buy, query.limit) {
                return Ok(warp::reply::json(&QueryResponse {
                    count: offers.len(),
                    offers,
                }));
            }

            Err(QueryEmptyResultError {}.into())
        }
        None => Err(QueryEmptyResultError {}.into()),
    }
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

fn with_store(
    store_map: Arc<StoreMap>,
) -> impl Filter<Extract = (Arc<StoreMap>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&store_map))
}

fn with_metrics(
    metrics: impl ApiMetrics + Clone + Send + std::fmt::Debug + 'static,
) -> impl Filter<Extract = (impl ApiMetrics + std::fmt::Debug,), Error = Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}
