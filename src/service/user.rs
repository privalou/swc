use anyhow::Error;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
#[async_trait]
pub trait UserApi {
    async fn get_user(&self, id: i32) -> Result<User, Error>;
    async fn create_user(&self, user: User) -> Result<String, Error>;
}

struct UserApiMongoAdapter {
    db: mongodb::Database,
}

#[async_trait]
impl UserApi for UserApiMongoAdapter {
    async fn get_user(&self, id: i32) -> Result<User, Error> {
        let collection = self.db.collection("users");
        let object_id = ObjectId::from_str(&id.to_string())?;
        let filter = doc! {"_id": object_id};
        let user = collection.find_one(filter, None).await?;
        Ok(user.expect("User not found"))
    }

    async fn create_user(&self, user: User) -> Result<String, Error> {
        let collection = self.db.collection("users");
        let user = collection.insert_one(user, None).await?;
        Ok(user.inserted_id.to_string())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<String>,

    pub first_name: Option<String>,

    pub email: Option<String>,

    pub default_currency: Option<String>,

    pub balance: Option<Vec<Balance>>,

    pub groups: Option<Vec<GroupBalance>>,

    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency_code: Option<String>,
    pub amount: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GroupBalance {
    pub group_id: Option<i64>,

    pub balance: Option<Vec<Balance>>,
}
