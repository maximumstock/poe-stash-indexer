use std::{convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use tracing::log::error;
use trade_common::league::League;
use warp::{reply::Json, Filter, Rejection, Reply};

use crate::{
    metrics::api::ApiMetrics,
    store::{Offer, Store},
};

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    sell: String,
    buy: String,
}

pub async fn init<T: Into<SocketAddr> + 'static>(
    options: T,
    metrics: impl ApiMetrics + Send + Sync + 'static,
    store: Arc<Store>,
) {
    let routes = healtcheck_endpoint()
        .or(search_endpoint(metrics, store))
        .with(warp::trace::request())
        .recover(error_handler);

    warp::serve(routes).bind(options).await
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    limit: Option<u32>,
    league: Option<String>,
}

fn search_endpoint(
    metrics: impl ApiMetrics + Send + 'static,
    store: Arc<Store>,
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
    store: Arc<Store>,
    mut metrics: impl ApiMetrics,
) -> Result<Json, Rejection> {
    metrics.inc_search_requests();

    let league = {
        match query.league {
            None => Ok(League::Challenge),
            Some(str) => League::from_str(&str).map_err(|_| QueryEmptyResultError {}),
        }
    }?;

    match store
        .fetch_offers(league, payload.sell, payload.buy, query.limit)
        .await
    {
        Ok(offers) => Ok(warp::reply::json(&QueryResponse::new(offers))),
        Err(e) => {
            error!("{:?}", e);
            Err(QueryEmptyResultError {}.into())
        }
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

fn healtcheck_endpoint() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(warp::path("healthcheck"))
        .map(|| "{\"health\": \"ok\"}")
}

fn with_store(
    store: Arc<Store>,
) -> impl Filter<Extract = (Arc<Store>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&store))
}

fn with_metrics(
    metrics: impl ApiMetrics + Send + 'static,
) -> impl Filter<Extract = (impl ApiMetrics,), Error = Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}
