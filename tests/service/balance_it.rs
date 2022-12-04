use swc::service::balance::{BalanceApi, BalanceApiMongoAdapter};
use testcontainers::{clients, images};

#[tokio::test]
async fn get_group_balance() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{host_port}/");
    let database = mongodb::Client::with_uri_str(url)
        .await
        .unwrap()
        .database("bot_test_db");
    let balance_service = BalanceApiMongoAdapter::new(database.clone());
    let res = balance_service
        .group_balance("111412".to_string())
        .await
        .unwrap();
    dbg!(res);
}
