use crate::route::with_client;
use mongodb::Client;
use warp::{Filter, Reply};

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("groups")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
        .and(with_client(client))
        .and_then(handlers::create_group)
}

mod handlers {
    use crate::service::group::{CreateGroupSpec, GroupApi, GroupApiMongoAdapter};
    use mongodb::Client;

    pub async fn create_group(
        create_group_spec: CreateGroupSpec,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("request body: {:?}", create_group_spec);
        let group = GroupApiMongoAdapter::new_with(client)
            .create_group(create_group_spec)
            .await
            .expect("Failed to create group");
        Ok(warp::reply::json(&group))
    }
}
