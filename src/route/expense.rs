use crate::route::with_client;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetExpensesRequest {
    group_id: String,
}

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let add_expense = warp::path!("expenses")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
        .and(with_client(client.clone()))
        .and_then(handlers::create_expense);

    let get_group_expenses = warp::path!("expenses")
        .and(warp::get())
        .and(warp::query::<GetExpensesRequest>())
        .and(with_client(client.clone()))
        .and_then(handlers::get_group_expenses);

    let update_expense = warp::path!("expenses" / String)
        .and(warp::put())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
        .and(with_client(client))
        .and_then(handlers::update_expense);

    add_expense.or(get_group_expenses).or(update_expense)
}

mod handlers {
    use crate::route::expense::GetExpensesRequest;
    use crate::service::expense::{
        CreateExpenseSpec, ExpenseApiMongoAdapter, ExpensesApi, ListExpensesRequest,
        UpdateExpenseSpec,
    };
    use mongodb::Client;

    pub async fn create_expense(
        create_expense_spec: CreateExpenseSpec,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("request body: {:?}", create_expense_spec);
        let expense = ExpenseApiMongoAdapter::new_with(client)
            .create_expense(create_expense_spec)
            .await
            .expect("Failed to create expense");
        Ok(warp::reply::json(&expense))
    }

    pub async fn update_expense(
        expense_id: String,
        update_expense_spec: UpdateExpenseSpec,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("request body: {:?}", update_expense_spec);
        let expense = ExpenseApiMongoAdapter::new_with(client)
            .update_expense(expense_id, update_expense_spec)
            .await
            .expect("Failed to update expense");
        Ok(warp::reply::json(&expense))
    }

    pub async fn get_group_expenses(
        request_param: GetExpensesRequest,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let expenses = ExpenseApiMongoAdapter::new_with(client)
            .list_expenses(ListExpensesRequest {
                group_id: Some(request_param.group_id),
                ..Default::default()
            })
            .await
            .expect("Failed to get group expenses");
        let json = warp::reply::json(&expenses);
        Ok(warp::reply::with_status(json, warp::http::StatusCode::OK))
    }
}
