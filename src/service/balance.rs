use anyhow::Error;
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Client};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait BalanceApi {
    async fn user_balance(&self, user_id: String) -> Result<Balance, Error>;
    async fn group_balance(&self, group_id: String) -> Result<Vec<Document>, Error>;
}

#[derive(Debug)]
pub struct BalanceApiMongoAdapter {
    db: mongodb::Database,
}

impl BalanceApiMongoAdapter {
    pub fn new(db: mongodb::Database) -> Self {
        Self { db }
    }
    pub fn new_with(client: Client) -> Self {
        Self::new(client.database("swc"))
    }
}

#[async_trait]
impl BalanceApi for BalanceApiMongoAdapter {
    async fn user_balance(&self, user_id: String) -> Result<Balance, Error> {
        let collection = self.db.collection::<Document>("balance");
        let filter = doc! {"user_id": user_id};
        let doc = collection.find_one(filter, None).await?.unwrap();
        let balance: Balance = bson::from_document(doc).unwrap();
        Ok(balance)
    }

    async fn group_balance(&self, group_id: String) -> Result<Vec<Document>, Error> {
        let collection = self.db.collection::<Document>("balance");
        let aggregation = vec![
            doc! {"$match": {"group_id": group_id}},
            doc! {
                "$group": {
                    "_id": "$group_id",
                    "user_id": {
                        "$push": {
                            "userId": "$user_id",
                            "paidShare": "$paidShare",
                            "owedShare": "$owedShare",
                            "netBalance": "$netBalance",
                        }
                    }
            }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "group_id": "$_id",
                    "user_id": 1,
                }
            },
        ];
        let cursor = collection.aggregate(aggregation, None).await?;
        let res = cursor.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub user_id: String,
    pub balance: i64,
}
