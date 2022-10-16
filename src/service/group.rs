use anyhow::Error;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio_stream::StreamExt;

#[async_trait]
pub trait GroupApi {
    async fn get_group(&self, id: i32) -> Result<Group, Error>;
    async fn create_group(&self, group: CreateGroupSpec) -> Result<Group, Error>;
    async fn get_user_group(&self, user_id: String) -> Result<Vec<Group>, Error>;
}

#[derive(Debug)]
pub struct GroupApiMongoAdapter {
    db: mongodb::Database,
}

impl GroupApiMongoAdapter {
    pub fn new(db: mongodb::Database) -> Self {
        Self { db }
    }

    pub fn new_with(client: Client) -> Self {
        Self::new(client.database("swc"))
    }
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

    async fn create_group(&self, create_spec: CreateGroupSpec) -> Result<Group, Error> {
        let collection = self.db.collection("groups");
        let members = create_spec
            .users
            .map(|users| users.into_iter().map(User::from).collect::<Vec<_>>());
        let group = Group {
            id: None,
            name: Some(create_spec.name),
            simplify_by_default: Some(true),
            members,
            ..Group::default()
        };
        let inserted_group = collection.insert_one(group.clone(), None).await?;
        let group = Group {
            id: Some(inserted_group.inserted_id.as_object_id().unwrap().to_hex()),
            ..group
        };
        Ok(group)
    }

    async fn get_user_group(&self, user_id: String) -> Result<Vec<Group>, Error> {
        let collection = self.db.collection::<mongodb::bson::Document>("groups");
        let filter = doc! {
            "members": {
                "$elemMatch": {
                    "user_id": user_id
                }
            }
        };
        let mut cursor = collection.find(filter, None).await?;

        let mut groups = Vec::new();
        while let Some(group) = cursor.try_next().await? {
            let group: Group = mongodb::bson::from_document(group).unwrap();
            groups.push(group);
        }
        Ok(groups)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub name: Option<String>,

    /// What is the group used for? One of:
    /// - `apartment`
    /// - `house`
    /// - `trip`
    /// - `other`
    pub group_type: Option<String>,

    /// Timestamp of when the group was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    pub simplify_by_default: Option<bool>,

    pub members: Option<Vec<User>>,

    pub original_debts: Option<Vec<Debt>>,

    pub simplified_debts: Option<Vec<Debt>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Option<String>,

    pub first_name: Option<String>,

    pub last_name: Option<String>,

    pub email: Option<String>,

    /// User's registration status. One of:
    /// - `confirmed`
    /// - `unconfirmed`
    pub registration_status: Option<String>,

    /// User's balance in each currency.
    pub balance: Option<Vec<Balance>>,
}

impl From<GroupUser> for User {
    fn from(group_user: GroupUser) -> User {
        Self {
            id: Some(group_user.user_id),
            first_name: group_user.first_name,
            ..User::default()
        }
    }
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
#[serde(rename_all = "camelCase")]
pub struct CreateGroupSpec {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<GroupUser>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub user_id: String,

    pub first_name: Option<String>,
}

#[cfg(test)]
mod test {
    #[test]
    fn should_convert_to_create_spec() {
        let json = r#"
            {
              "name": "group1",
              "users": [
                {
                  "userId": "user1",
                  "firstName": "John"
                }
              ]
            }
            "#;
        let spec: super::CreateGroupSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.name, "group1");
        assert_eq!(spec.users.as_ref().unwrap().len(), 1usize);
        let users = spec.users.unwrap();
        let first = users.first().unwrap();
        assert_eq!(&first.user_id, "user1");
        assert_eq!(first.first_name, Some("John".to_string()));
    }
}
