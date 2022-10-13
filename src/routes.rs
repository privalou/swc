use crate::service::group::{CreateGroupSpec, GroupApi, GroupApiMongoAdapter};
use crate::state::State;
use tide::{Request, Response};

/// Creates a new group. Adds the current user to the group by default.
/// Note that user id should be pa
///
///
pub async fn create_group(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let group_service = GroupApiMongoAdapter::new(req.state().mongo_client.database("group"));
    let group = req.body_json::<CreateGroupSpec>().await?;
    let group = group_service.create_group(group).await?;
    let response = Response::builder(200)
        .body(tide::Body::from_json(&group)?)
        .content_type(tide::http::mime::JSON)
        .build();
    Ok(response)
}
