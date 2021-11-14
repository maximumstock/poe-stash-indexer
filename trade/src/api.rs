use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use warp::{reply::Json, Filter, Rejection, Reply};

use crate::store::{Offer, Store};

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    sell: String,
    buy: String,
}

pub async fn init<T: Into<SocketAddr>>(options: T, store: Arc<Mutex<Store>>) {
    let routes = healtcheck_endpoint()
        .or(search_endpoint(store))
        .recover(error_handler);
    warp::serve(routes).run(options).await
}

fn search_endpoint(
    store: Arc<Mutex<Store>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("trade"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_store(store))
        .and_then(handle_search)
}

async fn handle_search(payload: RequestBody, store: Arc<Mutex<Store>>) -> Result<Json, Rejection> {
    let store = store.lock().await;
    let offers = store.query(&payload.sell, &payload.buy);

    if let Some(offers) = offers {
        return Ok(warp::reply::json(&QueryResponse { offers }));
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
    offers: Vec<&'a Offer>,
}

fn healtcheck_endpoint() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(warp::path("healthcheck"))
        .map(|| "{\"health\": \"ok\"}")
}

fn with_store(
    store: Arc<Mutex<Store>>,
) -> impl Filter<Extract = (Arc<Mutex<Store>>,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}
