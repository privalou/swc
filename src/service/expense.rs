use anyhow::Error;
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::{bson, Client, Database};
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
    async fn create_expense(&self, expense: CreateExpenseSpec) -> Result<ExpenseEntity, Error>;
    async fn update_expense(&self, id: String, spec: UpdateExpenseSpec) -> Result<Expense, Error>;
    async fn delete_expense(&self, id: i64) -> Result<(), Error>;
    async fn restore_expense(&self, id: i64) -> Result<(), Error>;
}

impl ExpenseApiMongoAdapter {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn new_with(client: Client) -> Self {
        Self::new(client.database("swc"))
    }

    fn build_update_expense_doc(update_expense_spec: UpdateExpenseSpec) -> Vec<Document> {
        let update_bson = bson::to_bson(&update_expense_spec).expect("Failed to serialize");
        let mut update_pipeline = Vec::new();
        update_pipeline.push(doc! {
            "$set": &update_bson
        });
        update_pipeline.push(doc! {
            "$set": {
                "updatedAt": bson::to_bson(&Utc::now()).expect("Failed to serialize")
            }
        });
        update_pipeline
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
                    "_id": ObjectId::from_str(&id).expect("Invalid id")
                },
                None,
            )
            .await?;
        let document = docs.unwrap();
        let expense: Result<Expense, _> = bson::from_document(document);
        Ok(expense.unwrap())
    }

    async fn list_expenses(&self, request: ListExpensesRequest) -> Result<ExpensesResponse, Error> {
        let mut cursor = self
            .db
            .collection("expenses")
            .find(
                doc! {
                    "groupId": request.group_id,
                    "deletedAt": doc! {
                        "$exists": false
                    }
                },
                None,
            )
            .await?;
        let mut expenses = Vec::new();
        while let Some(result) = cursor.next().await {
            let expense = result?;
            expenses.push(expense);
        }
        Ok(ExpensesResponse { expenses })
    }

    async fn create_expense(&self, expense: CreateExpenseSpec) -> Result<ExpenseEntity, Error> {
        let expense = ExpensesCalculator::new()
            .create_expense(&expense)
            .expect("Can not create expense");
        let (expense_document, option) = (bson::to_document(&expense)?, None);
        let expense_created = self
            .db
            .collection("expenses")
            .insert_one(expense_document, option)
            .await?;
        Ok(ExpenseEntity {
            id: Some(expense_created.inserted_id.as_object_id().unwrap()),
            expense,
        })
    }

    async fn update_expense(
        &self,
        id: String,
        update_expense_spec: UpdateExpenseSpec,
    ) -> Result<Expense, Error> {
        let document = Self::build_update_expense_doc(update_expense_spec);
        let update_result = self
            .db
            .collection::<Document>("expenses")
            .find_one_and_update(
                doc! {
                    "_id": ObjectId::from_str(&id).expect("Invalid id")
                },
                document,
                FindOneAndUpdateOptions::builder()
                    .return_document(Some(mongodb::options::ReturnDocument::After))
                    .build(),
            )
            .await?;

        Ok(
            bson::from_document(update_result.expect("Expense not found"))
                .expect("Can not deserialize expense"),
        )
    }

    async fn delete_expense(&self, _id: i64) -> Result<(), Error> {
        Ok(())
    }

    async fn restore_expense(&self, _id: i64) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub expense: Expense,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub cost: Option<String>,

    pub description: Option<String>,

    pub date: Option<DateTime<Utc>>,

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
    pub created_at: Option<DateTime<Utc>>,

    /// User that created the expense.
    pub created_by: Option<User>,

    /// The last time the expense was updated.
    pub updated_at: Option<DateTime<Utc>>,

    /// User that updated the expense.
    pub updated_by: Option<User>,

    /// If the expense was deleted, when it was deleted.
    pub deleted_at: Option<DateTime<Utc>>,

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListExpensesRequest {
    /// If provided, only expenses in that group will be returned, and
    /// `friend_id` will be ignored.
    pub group_id: Option<String>,

    /// ID of another user. If provided, only expenses between the current and
    /// provided user will be returned.
    pub friend_id: Option<String>,

    /// Filter to expenses after this date.
    pub dated_after: Option<DateTime<Utc>>,

    /// Filter to expenses before this date.
    pub dated_before: Option<DateTime<Utc>>,

    /// Filter to expenses updated after this date.
    pub updated_after: Option<DateTime<Utc>>,

    /// Filter to expenses updated before this date.
    pub updated_before: Option<DateTime<Utc>>,

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
        write!(formatter, "{}", format!("{self:?}").to_lowercase())
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
    pub date: Option<DateTime<Utc>>,

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

    pub group_id: Option<String>,

    /// Users by share if not splitting the expense equally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<UserShare>>,
}

/// User with share information associated with the expense.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserShare {
    pub user: Option<User>,

    pub paid_share: Option<String>,

    pub owed_share: Option<String>,

    pub net_balance: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Option<String>,

    pub first_name: Option<String>,

    pub email: Option<String>,

    pub default_currency: Option<String>,

    pub balance: Option<Vec<Balance>>,

    pub updated_at: Option<DateTime<Utc>>,
}

pub trait Expenses {
    fn create_expense(&self, create_expense_spec: &CreateExpenseSpec) -> Result<Expense, Error>;
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExpensesCalculator {}

impl ExpensesCalculator {
    pub fn new() -> Self {
        ExpensesCalculator {}
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShareCalculator {}

impl Expenses for ExpensesCalculator {
    fn create_expense(&self, create_expense_spec: &CreateExpenseSpec) -> Result<Expense, Error> {
        let user = create_expense_spec.user.clone();
        let payer_id = user.id.as_ref().expect("No user id");
        let share = ShareCalculator::new().equal_share(
            create_expense_spec.cost.clone(),
            payer_id.clone(),
            vec![payer_id.clone()],
        );
        Ok(Expense {
            cost: Some(create_expense_spec.cost.clone()),
            group_id: Some(create_expense_spec.group_id.parse()?),
            users: Some(share),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            created_by: Some(user),
            date: Some(Utc::now()),
            ..Expense::default()
        })
    }
}

impl ShareCalculator {
    pub fn new() -> Self {
        ShareCalculator {}
    }

    //noinspection RsBorrowChecker
    pub fn equal_share(
        &self,
        cost: String,
        payer_id: String,
        group: Vec<String>,
    ) -> Vec<UserShare> {
        if group.is_empty() {
            return vec![];
        }
        let cost = cost.parse::<f64>().unwrap();
        // we assume that the payer is a part of the group
        let common_share = cost / group.len() as f64;
        let payer_net = cost - common_share;

        let payer_share: UserShare = UserShare {
            user: Some(User {
                id: Some(payer_id.clone()),
                ..User::default()
            }),
            paid_share: Some(format!("{cost:.2}")),
            owed_share: Some(format!("{common_share:.2}")),
            net_balance: Some(format!("{payer_net:.2}")),
        };

        let debt_net = common_share * -1_f64;
        let remainder_share = group
            .iter()
            .filter(|&user_id| user_id != &payer_id)
            .map(|user_id| {
                let user_share: UserShare = UserShare {
                    user: Some(User {
                        id: Some(user_id.clone()),
                        ..User::default()
                    }),
                    paid_share: Some("0.00".to_string()),
                    owed_share: Some(format!("{common_share:.2}")),
                    net_balance: Some(format!("{debt_net:.2}")),
                };
                user_share
            })
            .chain(std::iter::once(payer_share))
            .collect::<Vec<UserShare>>();
        remainder_share
    }
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

    #[test]
    fn equally_split_expense() {
        use super::ShareCalculator;
        let calculator = ShareCalculator::default();
        let cost = "42.00".to_string();
        let payer_id = "1".to_string();
        let group = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let result = calculator.equal_share(cost, payer_id, group);
        assert_eq!(result.len(), 3);

        let user1 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("1".to_string()))
            .unwrap();
        assert_eq!(user1.paid_share, Some("42.00".to_string()));
        assert_eq!(user1.owed_share, Some("14.00".to_string()));
        assert_eq!(user1.net_balance, Some("28.00".to_string()));

        let user2 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("2".to_string()))
            .unwrap();
        assert_eq!(user2.paid_share, Some("0.00".to_string()));
        assert_eq!(user2.owed_share, Some("14.00".to_string()));
        assert_eq!(user2.net_balance, Some("-14.00".to_string()));

        let user3 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("3".to_string()))
            .unwrap();
        assert_eq!(user3.paid_share, Some("0.00".to_string()));
        assert_eq!(user3.owed_share, Some("14.00".to_string()));
        assert_eq!(user3.net_balance, Some("-14.00".to_string()));
    }

    #[test]
    fn single_user_group_share() {
        use super::ShareCalculator;
        let calculator = ShareCalculator::default();
        let cost = "42.00".to_string();
        let payer_id = "1".to_string();
        let group = vec!["1".to_string()];
        let result = calculator.equal_share(cost, payer_id, group);
        assert_eq!(result.len(), 1);

        let user1 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("1".to_string()))
            .unwrap();
        assert_eq!(user1.paid_share, Some("42.00".to_string()));
        assert_eq!(user1.owed_share, Some("42.00".to_string()));
        assert_eq!(user1.net_balance, Some("0.00".to_string()));
    }

    #[test]
    fn multiple_user_group_share() {
        use super::ShareCalculator;
        let calculator = ShareCalculator::default();
        let cost = "42.00".to_string();
        let payer_id = "1".to_string();
        let group = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ];
        let result = calculator.equal_share(cost, payer_id, group);
        assert_eq!(result.len(), 4);

        let user1 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("1".to_string()))
            .unwrap();
        assert_eq!(user1.paid_share, Some("42.00".to_string()));
        assert_eq!(user1.owed_share, Some("10.50".to_string()));
        assert_eq!(user1.net_balance, Some("31.50".to_string()));

        let user2 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("2".to_string()))
            .unwrap();
        assert_eq!(user2.paid_share, Some("0.00".to_string()));
        assert_eq!(user2.owed_share, Some("10.50".to_string()));
        assert_eq!(user2.net_balance, Some("-10.50".to_string()));

        let user3 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("3".to_string()))
            .unwrap();
        assert_eq!(user3.paid_share, Some("0.00".to_string()));
        assert_eq!(user3.owed_share, Some("10.50".to_string()));
        assert_eq!(user3.net_balance, Some("-10.50".to_string()));

        let user4 = result
            .iter()
            .find(|share| share.user.as_ref().unwrap().id == Some("4".to_string()))
            .unwrap();
        assert_eq!(user4.paid_share, Some("0.00".to_string()));
        assert_eq!(user4.owed_share, Some("10.50".to_string()));
        assert_eq!(user4.net_balance, Some("-10.50".to_string()));
    }

    #[test]
    fn create_expense() {
        use super::{CreateExpenseSpec, Expenses, ExpensesCalculator, User};
        let calculator = ExpensesCalculator::new();
        let expense = calculator
            .create_expense(&CreateExpenseSpec {
                cost: "42.00".to_string(),
                group_id: "1".to_string(),
                user: User {
                    id: Some("1".to_string()),
                    ..Default::default()
                },
            })
            .expect("Failed to create expense");
        assert_eq!(expense.cost, Some("42.00".to_string()));
        assert_eq!(expense.group_id, Some("1".to_string()));
        assert_eq!(
            expense.created_by.expect("Expected user").id,
            Some("1".to_string())
        );
        let shares = expense.users.expect("No Share Found For Expense");
        assert_eq!(shares.len(), 1);
        let share = shares
            .first()
            .expect("No Share Found For Expense")
            .to_owned();
        assert_eq!(share.user.expect("No User").id, Some("1".to_string()));
        assert_eq!(share.paid_share, Some("42.00".to_string()));
        assert_eq!(share.owed_share, Some("42.00".to_string()));
        assert_eq!(share.net_balance, Some("0.00".to_string()));
    }
}
