use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::expenses;

#[derive(Debug, Clone, diesel_derive_enum::DbEnum, Serialize, Deserialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::ExpenseStatus"]
pub enum ExpenseStatus {
    #[db_rename = "pending"]
    Pending,
    #[db_rename = "approved"]
    Approved,
    #[db_rename = "rejected"]
    Rejected,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = expenses)]
pub struct Expense {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub amount: i32,
    pub purpose: String,
    pub occurred_at: NaiveDate,
    pub note: Option<String>,
    pub receipt_file: Option<String>,
    pub status: ExpenseStatus,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = expenses)]
pub struct NewExpense {
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub amount: i32,
    pub purpose: String,
    pub occurred_at: NaiveDate,
    pub note: Option<String>,
    pub receipt_file: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = expenses)]
pub struct UpdateExpense {
    pub category_id: Option<Uuid>,
    pub amount: Option<i32>,
    pub purpose: Option<String>,
    pub occurred_at: Option<NaiveDate>,
    pub note: Option<String>,
    pub receipt_file: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = expenses)]
pub struct UpdateExpenseStatus {
    pub status: ExpenseStatus,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExpenseRequest {
    pub category_id: Option<Uuid>,
    pub amount: Option<i32>,
    pub purpose: Option<String>,
    pub occurred_at: Option<NaiveDate>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: ExpenseStatus,
}

#[derive(Debug, Deserialize)]
pub struct BulkApproveRequest {
    pub ids: Vec<Uuid>,
}
