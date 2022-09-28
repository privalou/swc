use anyhow::Error;
use async_trait::async_trait;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct ExpenseApiMongoAdapter {
    pub db: Database,
}

#[async_trait]
pub trait ExpensesApi {
    async fn get_expense(&self, id: String) -> Result<Expense, Error>;
    async fn list_expenses(&self, request: ListExpensesRequest) -> Result<ExpensesResponse, Error>;
    async fn create_expense(&self, expense: CreateExpenseSpec) -> Result<String, Error>;
    async fn update_expense(
        &self,
        id: String,
        spec: UpdateExpenseSpec,
    ) -> Result<Vec<Expense>, Error>;
    async fn delete_expense(&self, id: i64) -> Result<(), Error>;
    async fn restore_expense(&self, id: i64) -> Result<(), Error>;
}

impl ExpenseApiMongoAdapter {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExpensesApi for ExpenseApiMongoAdapter {
    async fn get_expense(&self, id: String) -> Result<Expense, Error> {
        let docs = self
            .db
            .collection::<Document>("expenses")
            .find_one(
                doc! {
                    "groupId": id.to_string(),
                },
                None,
            )
            .await?;
        let expense: Expense = bson::from_bson(bson::Bson::Document(docs.unwrap()))?;
        Ok(expense)
    }

    async fn list_expenses(
        &self,
        _request: ListExpensesRequest,
    ) -> Result<ExpensesResponse, Error> {
        let mut cursor = self.db.collection("expenses").find(None, None).await?;
        let mut expenses = Vec::new();
        while let Some(result) = cursor.next().await {
            let expense = result?;
            expenses.push(expense);
        }
        Ok(ExpensesResponse { expenses })
    }

    /// Create a new expense. expense is saved to the dedicated collection and record in the balance
    /// collection is updated
    async fn create_expense(&self, expense: CreateExpenseSpec) -> Result<String, Error> {
        let share = UserShare {
            user: Some(expense.user.clone()),
            paid_share: Some(expense.cost.clone()),
            owed_share: Some(expense.cost.clone()),
            net_balance: Some("0.0".to_string()),
            ..UserShare::default()
        };
        let expense = Expense {
            cost: Some(expense.cost),
            group_id: Some(expense.group_id),
            created_by: Some(expense.user),
            users: Some(vec![share]),
            ..Expense::default()
        };
        let (expense, option) = (bson::to_document(&expense)?, None);
        let expense_created = self
            .db
            .collection("expenses")
            .insert_one(expense, option)
            .await?;
        Ok(expense_created.inserted_id.as_object_id().unwrap().to_hex())
    }

    /*
     * Update expense by id only non-null fields
     */
    async fn update_expense(
        &self,
        id: String,
        update_expense_spec: UpdateExpenseSpec,
    ) -> Result<Vec<Expense>, Error> {
        let filter = doc! {
            "_id": ObjectId::from_str(&id).expect("Invalid id")
        };

        let set_bson = bson::to_bson(&update_expense_spec)?;

        let update = doc! {
            "$set": set_bson
        };

        let option = None;

        let _update_result = self
            .db
            .collection::<Document>("expenses")
            .update_one(filter, update, option)
            .await?;

        Ok(vec![])
    }

    async fn delete_expense(&self, _id: i64) -> Result<(), Error> {
        Ok(())
    }

    async fn restore_expense(&self, _id: i64) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub cost: Option<String>,

    pub description: Option<String>,

    pub date: Option<chrono::DateTime<Utc>>,

    pub repeat_interval: Option<RepeatInterval>,

    pub currency_code: Option<String>,

    /// Null if the expense is not associated with a group.
    pub group_id: Option<String>,

    pub repeats: Option<bool>,

    /// Whether this was a payment between users.
    pub payment: Option<bool>,

    /// Transaction method.
    pub transaction_method: Option<String>,

    /// Transaction status.
    pub transaction_status: Option<String>,

    /// List of debts between users.
    pub repayments: Option<Vec<Debt>>,

    /// The date and time the expense was created on Splitwise.
    pub created_at: Option<chrono::DateTime<Utc>>,

    /// User that created the expense.
    pub created_by: Option<User>,

    /// The last time the expense was updated.
    pub updated_at: Option<chrono::DateTime<Utc>>,

    /// User that updated the expense.
    pub updated_by: Option<User>,

    /// If the expense was deleted, when it was deleted.
    pub deleted_at: Option<chrono::DateTime<Utc>>,

    pub deleted_by: Option<User>,

    pub users: Option<Vec<UserShare>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Debt {
    pub from: Option<i64>,

    pub to: Option<i64>,

    pub amount: Option<String>,

    pub currency_code: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ExpensesResponse {
    pub expenses: Vec<Expense>,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ListExpensesRequest {
    /// If provided, only expenses in that group will be returned, and
    /// `friend_id` will be ignored.
    pub group_id: Option<i64>,

    /// ID of another user. If provided, only expenses between the current and
    /// provided user will be returned.
    pub friend_id: Option<i64>,

    /// Filter to expenses after this date.
    pub dated_after: Option<chrono::DateTime<Utc>>,

    /// Filter to expenses before this date.
    pub dated_before: Option<chrono::DateTime<Utc>>,

    /// Filter to expenses updated after this date.
    pub updated_after: Option<chrono::DateTime<Utc>>,

    /// Filter to expenses updated before this date.
    pub updated_before: Option<chrono::DateTime<Utc>>,

    /// Maximum number of expenses to return.
    /// Default: `20`
    pub limit: Option<i64>,

    /// Offset in the returned set of expenses.
    /// Default: `0`
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExpenseSpec {
    /// A string representation of a decimal value, limited to 2 decimal places.
    pub cost: String,

    pub group_id: String,

    pub user: User,
}

impl Default for CreateExpenseSpec {
    fn default() -> Self {
        Self {
            cost: "0.00".to_string(),
            group_id: "".to_string(),
            user: User::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RepeatInterval {
    Never,
    Weekly,
    Fortnightly,
    Monthly,
    Yearly,
}

impl fmt::Display for RepeatInterval {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GroupBalance {
    pub group_id: Option<i64>,

    pub balance: Option<Vec<Balance>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency_code: Option<String>,
    pub amount: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExpenseSpec {
    /// A string representation of a decimal value, limited to 2 decimal places.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// The date and time the expense took place. May differ from `created_at`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<chrono::DateTime<Utc>>,

    // TODO: Make this an enum
    /// Cadence at which the expense repeats. One of:
    /// - `never`
    /// - `weekly`
    /// - `fortnightly`
    /// - `monthly`
    /// - `yearly`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_interval: Option<String>,

    /// A currency code. Must be in the list from `get_currencies`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// A category id from `get_categories`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i64>,

    pub group_id: i64,

    /// Users by share if not splitting the expense equally.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<UserShare>>,
}

/// User with share information associated with the expense.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UserShare {
    pub user: Option<User>,

    pub user_id: Option<i64>,

    pub first_name: Option<String>,

    pub last_name: Option<String>,

    pub email: Option<String>,

    pub paid_share: Option<String>,

    pub owed_share: Option<String>,

    pub net_balance: Option<String>,
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

    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

mod test {
    #[test]
    fn display_repeat_interval() {
        use super::RepeatInterval;
        assert_eq!(format!("{}", RepeatInterval::Never), "never");
        assert_eq!(format!("{}", RepeatInterval::Weekly), "weekly");
        assert_eq!(format!("{}", RepeatInterval::Fortnightly), "fortnightly");
        assert_eq!(format!("{}", RepeatInterval::Monthly), "monthly");
        assert_eq!(format!("{}", RepeatInterval::Yearly), "yearly");
    }
}
