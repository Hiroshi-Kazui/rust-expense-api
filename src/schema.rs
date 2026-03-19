// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "expense_status"))]
    pub struct ExpenseStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExpenseStatus;
    use super::sql_types::UserRole;

    expenses (id) {
        id -> Uuid,
        user_id -> Uuid,
        category_id -> Uuid,
        amount -> Int4,
        purpose -> Text,
        occurred_at -> Date,
        note -> Nullable<Text>,
        receipt_file -> Nullable<Varchar>,
        status -> ExpenseStatus,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    categories (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        role -> UserRole,
        created_at -> Timestamp,
    }
}

diesel::joinable!(expenses -> users (user_id));
diesel::joinable!(expenses -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(expenses, users, categories,);
