use warp::{Filter, Rejection, Reply};

pub(crate) fn healtcheck_endpoint() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone
{
    warp::get()
        .and(warp::path("healthcheck"))
        .map(|| "{\"health\": \"ok\"}")
}
