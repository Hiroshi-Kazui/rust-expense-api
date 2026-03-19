// TC-MID-01〜03: Auth middleware integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use serial_test::serial;

/// TC-MID-01: Authorizationヘッダーなし
#[actix_rt::test]
#[serial]
async fn test_middleware_no_auth_header() {
    common::cleanup_db();
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/expenses")
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Authorization header missing"));
}

/// TC-MID-02: Bearer プレフィックスなし
#[actix_rt::test]
#[serial]
async fn test_middleware_no_bearer_prefix() {
    common::cleanup_db();
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/expenses")
        .insert_header(("Authorization", "notbearer sometoken"))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Bearer token required"));
}

/// TC-MID-03: 期限切れトークン
#[actix_rt::test]
#[serial]
async fn test_middleware_expired_token() {
    common::cleanup_db();
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct ExpiredClaims {
        sub: String,
        role: String,
        exp: i64,
        iat: i64,
    }

    let claims = ExpiredClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        role: "User".to_string(),
        exp: chrono::Utc::now().timestamp() - 3600,
        iat: chrono::Utc::now().timestamp() - 7200,
    };

    let secret = "test-secret-key-for-testing-32chars!!";
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).unwrap();

    let app = common::build_app().await;
    let req = test::TestRequest::get()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
