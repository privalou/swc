use mongodb::Client;
use swc::route::routes;
use swc::service::expense::{CreateExpenseSpec, User};
use testcontainers::{clients, images};
use warp::test::request;

#[tokio::test]
async fn create_expense() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{host_port}/");
    let client = Client::with_uri_str(url)
        .await
        .expect("Failed to connect to mongo");
    let create_expense_spec = CreateExpenseSpec {
        cost: "30.00".to_string(),
        group_id: "1234".to_string(),
        user: User {
            id: Some("1234".to_string()),
            ..User::default()
        },
    };
    let res = request()
        .method("POST")
        .path("/expenses")
        .json(&create_expense_spec)
        .reply(&routes(client))
        .await;
    assert_eq!(res.status(), 200);
}
