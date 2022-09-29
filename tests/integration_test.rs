use anyhow::Error;
use swc::service::expense::ExpenseApiMongoAdapter;
use swc::service::user::{CreateUserSpec, UserApi, UserApiMongoAdapter};
use testcontainers::{clients, images};

mod service {
    mod expense_it;
}

#[tokio::test]
async fn calculation_split_equally_for_three_users() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{}/", host_port.to_string());
    let database = mongodb::Client::with_uri_str(url)
        .await
        .unwrap()
        .database("bot_test_db");

    let _expense_service = ExpenseApiMongoAdapter::new(database.clone());
    let user_service = UserApiMongoAdapter::new(database.clone());

    let created_users_id = create_users(3, &user_service).await.unwrap();
    dbg!(&created_users_id);
}

async fn create_users(
    count: i32,
    user_service: &UserApiMongoAdapter,
) -> Result<Vec<String>, Error> {
    let mut users = Vec::new();
    for i in 0..count {
        let user = user_service
            .create_user(CreateUserSpec {
                first_name: format!("test{}", i),
                ..CreateUserSpec::default()
            })
            .await?;
        users.push(user);
    }
    Ok(users)
}
