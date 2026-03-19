// TC-AUTH-01〜05: Authentication integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use serial_test::serial;
use serde_json::{json, Value};

/// TC-AUTH-01: ログイン成功
#[actix_rt::test]
#[serial]
async fn test_login_success() {
    common::cleanup_db();
    let _user = common::create_user("admin@example.com", rust_expense_api::models::user::UserRole::Admin);

    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({"email": "admin@example.com", "password": "testpass"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "admin@example.com");
    assert_eq!(body["user"]["role"], "Admin");
    // password_hash must not be in response (it's skipped via serde)
    assert!(body["user"]["password_hash"].is_null());
}

/// TC-AUTH-02: メールアドレス不一致
#[actix_rt::test]
#[serial]
async fn test_login_wrong_email() {
    common::cleanup_db();
    common::create_user("admin@example.com", rust_expense_api::models::user::UserRole::Admin);

    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({"email": "notexist@example.com", "password": "testpass"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Invalid email or password"));
}

/// TC-AUTH-03: パスワード不一致
#[actix_rt::test]
#[serial]
async fn test_login_wrong_password() {
    common::cleanup_db();
    common::create_user("admin@example.com", rust_expense_api::models::user::UserRole::Admin);

    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({"email": "admin@example.com", "password": "wrongpass"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// TC-AUTH-04: リクエストボディ不正（passwordなし）
#[actix_rt::test]
#[serial]
async fn test_login_missing_fields() {
    common::cleanup_db();
    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({"email": "admin@example.com"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

/// TC-AUTH-05: Content-Type なし（JSON以外）
#[actix_rt::test]
#[serial]
async fn test_login_wrong_content_type() {
    common::cleanup_db();
    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .insert_header(("Content-Type", "text/plain"))
        .set_payload("email=admin&password=changeme")
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
