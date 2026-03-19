// TC-USER-01〜07: User management integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use rust_expense_api::models::user::UserRole;
use serial_test::serial;
use serde_json::{json, Value};

/// TC-USER-01: ユーザー登録 — 正常系（Admin）
#[actix_rt::test]
#[serial]
async fn test_create_user_as_admin() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "name": "田中太郎",
            "email": "tanaka@example.com",
            "password": "pass1234",
            "role": "User"
        }))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "田中太郎");
    assert_eq!(body["email"], "tanaka@example.com");
    assert_eq!(body["role"], "User");
    assert!(body["id"].is_string());
    assert!(body["password_hash"].is_null());
}

/// TC-USER-02: ユーザー登録 — ロールのデフォルト値
#[actix_rt::test]
#[serial]
async fn test_create_user_default_role() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "name": "花子",
            "email": "hanako@example.com",
            "password": "pass"
        }))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["role"], "User");
}

/// TC-USER-03: ユーザー登録 — Email重複
#[actix_rt::test]
#[serial]
async fn test_create_user_duplicate_email() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    common::create_user("tanaka@example.com", UserRole::User);
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "name": "田中",
            "email": "tanaka@example.com",
            "password": "pass"
        }))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Email already exists"));
}

/// TC-USER-04: ユーザー登録 — 認証なし
#[actix_rt::test]
#[serial]
async fn test_create_user_no_auth() {
    common::cleanup_db();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(json!({"name": "x", "email": "x@x.com", "password": "x"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// TC-USER-05: ユーザー登録 — Userロールによるアクセス
#[actix_rt::test]
#[serial]
async fn test_create_user_as_non_admin() {
    common::cleanup_db();
    let (_user, token) = common::create_regular_user();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"name": "x", "email": "x2@x.com", "password": "x"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

/// TC-USER-06: ユーザー一覧取得 — Admin
#[actix_rt::test]
#[serial]
async fn test_list_users_as_admin() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    common::create_user("user1@test.com", UserRole::User);
    common::create_user("user2@test.com", UserRole::User);
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().len() >= 2);
    // No password_hash
    for user in body.as_array().unwrap() {
        assert!(user["password_hash"].is_null());
    }
}

/// TC-USER-07: ユーザー一覧取得 — 非Admin
#[actix_rt::test]
#[serial]
async fn test_list_users_as_non_admin() {
    common::cleanup_db();
    let (_user, token) = common::create_regular_user();
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}
