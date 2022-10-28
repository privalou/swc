use crate::route::with_client;
use crate::service::expense::CreateExpenseSpec;
use mongodb::Client;
use warp::Filter;

pub fn expenses(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("expenses")
        .and(warp::post())
        .and(json_body())
        .and(with_client(client))
        .and_then(handlers::create_expense)
}

fn json_body() -> impl Filter<Extract = (CreateExpenseSpec,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

mod handlers {

    use crate::service::expense::{CreateExpenseSpec, ExpenseApiMongoAdapter, ExpensesApi};
    use mongodb::Client;

    pub async fn create_expense(
        create_expense_spec: CreateExpenseSpec,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let expense = ExpenseApiMongoAdapter::new_with(client)
            .create_expense(create_expense_spec)
            .await
            .expect("Failed to create expense");
        Ok(warp::reply::json(&expense))
    }
}
