use crate::service::expense::{Balance, GroupBalance};
use serde::{Deserialize, Serialize};

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
