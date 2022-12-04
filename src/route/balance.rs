use crate::route::with_client;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListBalancesRequest {
    group_id: String,
}

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("balances")
        .and(warp::get())
        .and(warp::query::<ListBalancesRequest>())
        .and(with_client(client))
        .and_then(handlers::get_balances)
}

mod handlers {
    use crate::route::balance::ListBalancesRequest;
    use crate::service::balance::{BalanceApi, BalanceApiMongoAdapter};
    use mongodb::Client;

    pub async fn get_balances(
        request_param: ListBalancesRequest,
        client: Client,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let balances = BalanceApiMongoAdapter::new_with(client)
            .group_balance(request_param.group_id)
            .await
            .expect("Failed to get group balances");
        let json = warp::reply::json(&balances);
        Ok(warp::reply::with_status(json, warp::http::StatusCode::OK))
    }
}
