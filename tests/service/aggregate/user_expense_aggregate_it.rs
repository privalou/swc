use anyhow::Error;
use swc::service::expense::{CreateExpenseSpec, ExpenseApiMongoAdapter, ExpensesApi};
use swc::service::user::{CreateUserSpec, UserApi, UserApiMongoAdapter};
use testcontainers::{clients, images};

#[tokio::test]
async fn calculation_split_equally_for_three_users() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{}/", host_port);
    let database = mongodb::Client::with_uri_str(url)
        .await
        .unwrap()
        .database("bot_test_db");

    let user_service = UserApiMongoAdapter::new(database.clone());

    let _created_users_id = create_users(3, &user_service).await.unwrap();

    let expense_service = ExpenseApiMongoAdapter::new(database.clone());
    let created_expense = expense_service
        .create_expense(CreateExpenseSpec {
            cost: "100".to_string(),
            group_id: "1".to_string(),
            user: swc::service::expense::User {
                id: Some("1".to_string()),
                first_name: Some("test".to_string()),
                ..swc::service::expense::User::default()
            },
        })
        .await
        .unwrap();

    let expense = expense_service
        .get_expense(created_expense.id.unwrap().to_hex())
        .await
        .unwrap();

    assert_eq!(expense.cost, Some("100".to_string()));

    let update_expense_spec = swc::service::expense::UpdateExpenseSpec {
        cost: Some("30".to_string()),
        ..swc::service::expense::UpdateExpenseSpec::default()
    };

    expense_service
        .update_expense(created_expense.id.unwrap().to_hex(), update_expense_spec)
        .await
        .unwrap();

    let updated_expense = expense_service
        .get_expense(created_expense.id.unwrap().to_hex())
        .await
        .unwrap();

    assert_eq!(updated_expense.cost, Some("30".to_string()));
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
