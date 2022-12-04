mod balance;
mod expense;
mod group;

use mongodb::Client;
use serde::Serialize;
use warp::{Filter, Rejection};

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    group::routes(client.clone())
        .or(expense::routes(client.clone()))
        .or(balance::routes(client))
        .or(health())
}

fn health() -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
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

#[derive(Debug, Clone, Serialize)]
struct ErrorMessage {
    message: String,
}
