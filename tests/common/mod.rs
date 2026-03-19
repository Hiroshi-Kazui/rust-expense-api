use actix_web::web;
use diesel::prelude::*;
use once_cell::sync::Lazy;
use rust_expense_api::{
    auth::jwt::generate_token,
    config::Config,
    db::{self, Pool},
    handlers,
    models::{
        category::{Category, NewCategory},
        expense::{Expense, ExpenseStatus, NewExpense},
        user::{NewUser, User, UserRole},
    },
    schema::{categories, expenses, users},
    MIGRATIONS,
};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

pub fn test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://expense_user:expense_pass@localhost:5432/expense_test".to_string()
    })
}

pub fn test_config() -> Config {
    Config {
        database_url: test_db_url(),
        jwt_secret: "test-secret-key-for-testing-32chars!!".to_string(),
        jwt_expires_in_hours: 24,
        upload_dir: "./test_uploads".to_string(),
    }
}

static TEST_POOL: Lazy<Pool> = Lazy::new(|| {
    let url = test_db_url();
    let pool = db::create_pool(&url);
    // Run migrations once
    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run test migrations");
    pool
});

pub fn get_pool() -> Pool {
    TEST_POOL.clone()
}

/// Delete all rows in dependency order to avoid FK violations
pub fn cleanup_db() {
    let pool = get_pool();
    let mut conn = pool.get().unwrap();
    diesel::delete(expenses::table).execute(&mut conn).unwrap();
    diesel::delete(categories::table).execute(&mut conn).unwrap();
    diesel::delete(users::table).execute(&mut conn).unwrap();
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::auth::login)
        .service(handlers::users::create_user)
        .service(handlers::users::list_users)
        .service(handlers::categories::create_category)
        .service(handlers::categories::list_categories)
        .service(handlers::expenses::bulk_approve)
        .service(handlers::expenses::create_expense)
        .service(handlers::expenses::list_expenses)
        .service(handlers::expenses::update_expense)
        .service(handlers::expenses::delete_expense)
        .service(handlers::expenses::update_expense_status);
}

/// Call the test app with a request and return the response.
/// This helper avoids type inference issues with `impl Service<...>`.
pub async fn call(req: actix_web::test::TestRequest) -> actix_web::dev::ServiceResponse {
    let pool = get_pool();
    let cfg = test_config();
    let pool_data = web::Data::new(pool);
    let cfg_data = web::Data::new(cfg);

    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(pool_data)
            .app_data(cfg_data)
            .app_data(
                web::JsonConfig::default().error_handler(|err, _| {
                    let response =
                        rust_expense_api::errors::AppError::BadRequest(err.to_string());
                    actix_web::error::InternalError::from_response(
                        err,
                        actix_web::ResponseError::error_response(&response),
                    )
                    .into()
                }),
            )
            .configure(configure_app),
    )
    .await;

    actix_web::test::call_service(&app, req.to_request()).await
}

/// Build the actix-web application for testing.
pub async fn build_app(
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let pool = get_pool();
    let cfg = test_config();
    let pool_data = web::Data::new(pool);
    let cfg_data = web::Data::new(cfg);

    actix_web::test::init_service(
        actix_web::App::new()
            .app_data(pool_data)
            .app_data(cfg_data)
            .app_data(
                web::JsonConfig::default().error_handler(|err, _| {
                    let response =
                        rust_expense_api::errors::AppError::BadRequest(err.to_string());
                    actix_web::error::InternalError::from_response(
                        err,
                        actix_web::ResponseError::error_response(&response),
                    )
                    .into()
                }),
            )
            .configure(configure_app),
    )
    .await
}

/// Create a user directly in DB (uses cost=4 for bcrypt speed in tests)
pub fn create_user(email: &str, role: UserRole) -> User {
    let pool = get_pool();
    let mut conn = pool.get().unwrap();
    // Use cost 4 (minimum valid bcrypt cost) for fast test execution
    let hash = bcrypt::hash("testpass", 4).unwrap();
    let new_user = NewUser {
        name: format!("Test {}", email),
        email: email.to_string(),
        password_hash: hash,
        role,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .unwrap()
}

/// Get JWT for a user
pub fn get_token(user: &User) -> String {
    let cfg = test_config();
    generate_token(user.id, user.role.clone(), &cfg.jwt_secret, cfg.jwt_expires_in_hours).unwrap()
}

/// Create admin user + token
pub fn create_admin() -> (User, String) {
    let user = create_user("admin@test.com", UserRole::Admin);
    let token = get_token(&user);
    (user, token)
}

/// Create regular user + token
pub fn create_regular_user() -> (User, String) {
    let suffix = Uuid::new_v4().to_string()[..8].to_string();
    let email = format!("user_{}@test.com", suffix);
    let user = create_user(&email, UserRole::User);
    let token = get_token(&user);
    (user, token)
}

/// Create a category
pub fn create_category(name: &str) -> Category {
    let pool = get_pool();
    let mut conn = pool.get().unwrap();
    diesel::insert_into(categories::table)
        .values(NewCategory { name: name.to_string() })
        .returning(Category::as_returning())
        .get_result(&mut conn)
        .unwrap()
}

/// Create an expense with given status for a user
pub fn create_expense(user_id: Uuid, category_id: Uuid, target_status: ExpenseStatus) -> Expense {
    use chrono::NaiveDate;
    let pool = get_pool();
    let mut conn = pool.get().unwrap();
    let new_exp = NewExpense {
        user_id,
        category_id,
        amount: 1000,
        purpose: "テスト経費".to_string(),
        occurred_at: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        note: None,
        receipt_file: None,
    };
    let expense: Expense = diesel::insert_into(expenses::table)
        .values(&new_exp)
        .returning(Expense::as_returning())
        .get_result(&mut conn)
        .unwrap();

    if target_status != ExpenseStatus::Pending {
        diesel::update(expenses::table.find(expense.id))
            .set(expenses::status.eq(target_status))
            .returning(Expense::as_returning())
            .get_result(&mut conn)
            .unwrap()
    } else {
        expense
    }
}

/// Minimal valid 1x1 JPEG bytes
pub fn minimal_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
        0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
        0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
        0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
        0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
        0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
        0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
        0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00,
        0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
        0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06,
        0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
        0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72,
        0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
        0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45,
        0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
        0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75,
        0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
        0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3,
        0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
        0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9,
        0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
        0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4,
        0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01,
        0x00, 0x00, 0x3F, 0x00, 0xFB, 0xD3, 0xFF, 0xD9,
    ]
}
