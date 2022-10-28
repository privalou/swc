mod expense;
mod group;

use mongodb::Client;
use warp::Filter;

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    group::groups(client.clone())
        .or(expense::expenses(client))
        .or(health())
}

fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("health")
        .and(warp::get())
        .map(|| warp::http::StatusCode::OK)
        .with(warp::cors().allow_any_origin())
}

fn with_client(
    client: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}
