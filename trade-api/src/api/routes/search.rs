use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tracing::error;
use trade_common::league::League;
use warp::{reply::Json, Filter, Rejection, Reply};

use crate::{
    api::{
        middleware::{with_metrics, with_store},
        QueryEmptyResultError, QueryResponse,
    },
    metrics::api::ApiMetrics,
    store::{Store, StoreQuery},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    sell: Option<String>,
    buy: Option<String>,
    seller_account: Option<String>,
    stash_id: Option<String>,
    league: Option<String>,
    limit: Option<u32>,
}

pub(crate) fn search(
    metrics: impl ApiMetrics + Send + 'static,
    store: Arc<Store>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("trade"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_store(store))
        .and(with_metrics(metrics))
        .and_then(handle_search)
}

#[tracing::instrument(skip(store, metrics))]
async fn handle_search(
    payload: RequestBody,
    store: Arc<Store>,
    mut metrics: impl ApiMetrics,
) -> Result<Json, Rejection> {
    metrics.inc_search_requests();

    let league = {
        match &payload.league {
            None => Ok(League::Challenge),
            Some(str) => League::from_str(str).map_err(|_| QueryEmptyResultError {}),
        }
    }?;

    let query = StoreQuery {
        sell: payload.sell,
        buy: payload.buy,
        seller_account: payload.seller_account,
        stash_id: payload.stash_id,
        limit: payload.limit,
    };

    match store.query(league, query).await {
        Ok(offers) => Ok(warp::reply::json(&QueryResponse::new(offers))),
        Err(e) => {
            error!("{:?}", e);
            Err(QueryEmptyResultError::new().into())
        }
    }
}
