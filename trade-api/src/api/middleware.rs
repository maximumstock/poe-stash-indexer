use std::{convert::Infallible, sync::Arc};

use warp::Filter;

use crate::{metrics::api::ApiMetrics, store::Store};

pub(crate) fn with_store(
    store: Arc<Store>,
) -> impl Filter<Extract = (Arc<Store>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&store))
}

pub(crate) fn with_metrics(
    metrics: impl ApiMetrics + Send + 'static,
) -> impl Filter<Extract = (impl ApiMetrics,), Error = Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}
