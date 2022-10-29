use crate::route::with_client;
use crate::service::group::CreateGroupSpec;
use mongodb::Client;
use warp::Filter;

pub fn groups(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("groups")
        .and(warp::post())
        .and(json_body())
        .and(with_client(client))
        .and_then(handlers::create_group)
}

fn json_body() -> impl Filter<Extract = (CreateGroupSpec,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
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
