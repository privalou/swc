use crate::service::group::CreateGroupSpec;
use mongodb::Client;
use warp::Filter;

pub fn filters(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    groups(client).or(health())
}

fn groups(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("groups")
        .and(warp::post())
        .and(json_body())
        .and(with_client(client))
        .and_then(handlers::create_group)
}

fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("health")
        .and(warp::get())
        .map(|| warp::http::StatusCode::OK)
        .with(warp::cors().allow_any_origin())
}

fn json_body() -> impl Filter<Extract = (CreateGroupSpec,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn with_client(
    client: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}

mod handlers {
    use crate::service::group::{CreateGroupSpec, GroupApi, GroupApiMongoAdapter};
    use mongodb::Client;

    pub async fn create_group(
        create_group_spec: CreateGroupSpec,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let group = GroupApiMongoAdapter::new_with(client)
            .create_group(create_group_spec)
            .await
            .expect("Failed to create group");
        Ok(warp::reply::json(&group))
    }
}
