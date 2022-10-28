use mongodb::Client;
use swc::route::routes;
use swc::service::group::{CreateGroupSpec, GroupUser};
use testcontainers::{clients, images};
use warp::test::request;

#[tokio::test]
async fn test_create_group() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = Client::with_uri_str(url)
        .await
        .expect("Failed to connect to mongo");
    let create_group_spec = CreateGroupSpec {
        name: "Test Group".to_string(),
        users: Some(vec![GroupUser::default()]),
    };
    let res = request()
        .method("POST")
        .path("/groups")
        .json(&create_group_spec)
        .reply(&routes(client))
        .await;
    assert_eq!(res.status(), 200);
}
