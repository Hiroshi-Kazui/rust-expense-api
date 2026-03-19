// TC-JWT-01〜06: JWT unit tests

use rust_expense_api::auth::jwt::{generate_token, verify_token};
use rust_expense_api::errors::AppError;
use rust_expense_api::models::user::UserRole;
use uuid::Uuid;

fn test_user_id() -> Uuid {
    Uuid::new_v4()
}

const SECRET: &str = "test-secret-key-for-testing-32chars!!";

/// TC-JWT-01: トークン生成 — 正常系
#[test]
fn test_generate_token_success() {
    let id = test_user_id();
    let result = generate_token(id, UserRole::User, SECRET, 24);
    assert!(result.is_ok());
    let token = result.unwrap();
    // JWT has 3 parts separated by '.'
    assert_eq!(token.split('.').count(), 3);
}

/// TC-JWT-02: トークン検証 — 正常系
#[test]
fn test_verify_token_success() {
    let id = test_user_id();
    let token = generate_token(id, UserRole::User, SECRET, 24).unwrap();
    let claims = verify_token(&token, SECRET).unwrap();
    assert_eq!(claims.sub, id);
}

/// TC-JWT-03: トークン検証 — 署名不正
#[test]
fn test_verify_token_wrong_secret() {
    let id = test_user_id();
    let token = generate_token(id, UserRole::User, SECRET, 24).unwrap();
    let result = verify_token(&token, "wrong-secret-key-for-testing-32!!!");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
}

/// TC-JWT-04: トークン検証 — 期限切れ
#[test]
fn test_verify_token_expired() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct ExpiredClaims {
        sub: String,
        exp: i64,
        iat: i64,
    }

    let claims = ExpiredClaims {
        sub: Uuid::new_v4().to_string(),
        exp: chrono::Utc::now().timestamp() - 3600, // 1 hour ago
        iat: chrono::Utc::now().timestamp() - 7200,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap();

    let result = verify_token(&token, SECRET);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
}

/// TC-JWT-05: トークン検証 — 不正形式
#[test]
fn test_verify_token_malformed() {
    let result = verify_token("not.a.jwt", SECRET);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
}

/// TC-JWT-06: Claims のロールが保持される
#[test]
fn test_token_preserves_admin_role() {
    let id = test_user_id();
    let token = generate_token(id, UserRole::Admin, SECRET, 24).unwrap();
    let claims = verify_token(&token, SECRET).unwrap();
    assert_eq!(claims.role, UserRole::Admin);
}
