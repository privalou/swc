use async_trait::async_trait;
use anyhow::Error;
use mongodb::bson;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait BalanceApi {
    async fn get_user_balance(&self, user_id: String) -> Result<Balance, Error>;
}


#[derive(Debug)]
pub struct BalanceApiMongoAdapter {
    db: mongodb::Database,
}

#[async_trait]
impl BalanceApi for BalanceApiMongoAdapter {
    async fn get_user_balance(&self, user_id: String) -> Result<Balance, Error> {
        let collection = self.db.collection::<Document>("balance");
        let filter = doc! {"user_id": user_id};
        let doc = collection.find_one(filter, None).await?.unwrap();
        let balance: Balance  = bson::from_document(doc).unwrap();
        Ok(balance)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub user_id: String,
    pub balance: i64,
}
