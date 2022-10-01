use anyhow::Error;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[async_trait]
pub trait GroupApi {
    async fn get_group(&self, id: i32) -> Result<Group, Error>;
    async fn create_group(&self, group: CreateGroupSpec) -> Result<String, Error>;
}

#[derive(Debug)]
pub struct GroupApiMongoAdapter {
    db: mongodb::Database,
}

#[async_trait]
impl GroupApi for GroupApiMongoAdapter {
    async fn get_group(&self, id: i32) -> Result<Group, Error> {
        let collection = self.db.collection("groups");
        let object_id = ObjectId::from_str(&id.to_string())?;
        let filter = doc! {"_id": object_id};
        let group = collection.find_one(filter, None).await?;
        Ok(group.expect("Group not found"))
    }

    async fn create_group(&self, create_spec: CreateGroupSpec) -> Result<String, Error> {
        let collection = self.db.collection("groups");
        let group = Group {
            id: None,
            name: Some(create_spec.name),
            ..Group::default()
        };
        let group = collection.insert_one(group, None).await?;
        Ok(group.inserted_id.to_string())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Group name.
    pub name: Option<String>,

    /// What is the group used for? One of:
    /// - `apartment`
    /// - `house`
    /// - `trip`
    /// - `other`
    pub group_type: Option<String>,

    /// Timestamp of when the group was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Turn on simplify debts?
    pub simplify_by_default: Option<bool>,

    /// List of users that are members of the group.
    pub members: Option<Vec<User>>,

    /// List of debts between users in the group.
    pub original_debts: Option<Vec<Debt>>,

    /// List of simplified debts between users in the group.
    pub simplified_debts: Option<Vec<Debt>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// User ID.
    pub id: Option<i64>,

    /// User's first name.
    pub first_name: Option<String>,

    /// User's last name.
    pub last_name: Option<String>,

    /// User's email address.
    pub email: Option<String>,

    /// User's registration status. One of:
    /// - `confirmed`
    /// - `unconfirmed`
    pub registration_status: Option<String>,

    /// User's balance in each currency.
    pub balance: Option<Vec<Balance>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency_code: Option<String>,
    pub amount: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Debt {
    pub from: Option<i64>,

    pub to: Option<i64>,

    pub amount: Option<String>,

    pub currency_code: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateGroupSpec {
    /// Group name.
    pub name: String,

    /// List of users to invite to the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<GroupUser>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupUser {
    pub user_id: Option<i64>,

    pub first_name: Option<String>,
}