use swc::service::expense::{CreateExpenseSpec, ExpensesApi, User, ExpensesService, UpdateExpenseSpec};
use futures::StreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use std::str::FromStr;
use testcontainers::{clients, images};

#[tokio::test]
async fn create_new_transaction() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{}/", host_port.to_string());
    let database = mongodb::Client::with_uri_str(url)
        .await
        .unwrap()
        .database("bot_test_db");

    let expense_service = ExpensesService::new(database.clone());

    expense_service
        .create_expense(CreateExpenseSpec {
            cost: "100".to_string(),
            group_id: "1".to_string(),
            user: User {
                id: Some("1".to_string()),
                first_name: Some("test".to_string()),
                ..User::default()
            },
            ..CreateExpenseSpec::default()
        })
        .await
        .unwrap();

    let mut cursor = database
        .clone()
        .collection::<Document>("expenses")
        .find(None, None)
        .await
        .unwrap();
    let mut expenses = Vec::new();
    while let Some(result) = cursor.next().await {
        let expense = result.unwrap();
        expenses.push(expense);
    }
    assert_eq!(expenses.len(), 1);
    let first = expenses.first().unwrap();
    assert_eq!(first.get_str("cost").unwrap(), "100");
    assert_eq!(first.get_str("groupId").unwrap(), "1");
    assert_eq!(
        first
            .get_document("createdBy")
            .unwrap()
            .get_str("firstName")
            .unwrap(),
        "test"
    );
}

#[tokio::test]
async fn update_only_non_none_fields_of_expense() {
    let docker = clients::Cli::default();
    let node = docker.run(images::mongo::Mongo::default());
    let host_port = node.get_host_port_ipv6(27017);
    let url = format!("mongodb://localhost:{}/", host_port.to_string());
    let database = mongodb::Client::with_uri_str(url)
        .await
        .unwrap()
        .database("bot_test_db");

    let expense_service = ExpensesService::new(database.clone());

    let id = expense_service
        .create_expense(CreateExpenseSpec {
            cost: "100".to_string(),
            group_id: "1".to_string(),
            user: User {
                id: Some("1".to_string()),
                first_name: Some("test".to_string()),
                ..User::default()
            },
            ..CreateExpenseSpec::default()
        })
        .await
        .unwrap();

    let object_id = ObjectId::from_str(id.as_str()).expect("Failed to parse object id");

    expense_service
        .update_expense(
            id.clone(),
            UpdateExpenseSpec {
                description: Some("test".to_string()),
                ..UpdateExpenseSpec::default()
            },
        )
        .await
        .unwrap();

    let mut cursor = database
        .clone()
        .collection::<Document>("expenses")
        .find(
            doc! {
                "_id": object_id
            },
            None,
        )
        .await
        .unwrap();

    let el = cursor.next().await.unwrap().unwrap();
    assert_eq!(el.get_str("description").unwrap(), "test");
}
