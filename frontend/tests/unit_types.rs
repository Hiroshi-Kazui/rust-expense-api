// TC-TYPE-01 through TC-TYPE-05
// Tests for types.rs: Display impls and serde deserialization.
// These are pure Rust logic tests that do not require WASM browser APIs,
// but they are compiled for the wasm32 target via wasm-bindgen-test so they
// run consistently alongside the WASM-specific tests.

use wasm_bindgen_test::wasm_bindgen_test;
use frontend::types::{Category, Expense, ExpenseStatus, LoginResponse, UserRole};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// TC-TYPE-01: ExpenseStatus Display
// ---------------------------------------------------------------------------

/// TC-TYPE-01: `ExpenseStatus::Display` returns the correct Japanese labels.
#[wasm_bindgen_test]
fn test_expense_status_display() {
    assert_eq!(
        format!("{}", ExpenseStatus::Pending),
        "申請中",
        "Pending should display as 申請中"
    );
    assert_eq!(
        format!("{}", ExpenseStatus::Approved),
        "承認済",
        "Approved should display as 承認済"
    );
    assert_eq!(
        format!("{}", ExpenseStatus::Rejected),
        "却下",
        "Rejected should display as 却下"
    );
}

// ---------------------------------------------------------------------------
// TC-TYPE-02: UserRole Display
// ---------------------------------------------------------------------------

/// TC-TYPE-02: `UserRole::Display` returns ASCII labels.
#[wasm_bindgen_test]
fn test_user_role_display() {
    assert_eq!(format!("{}", UserRole::Admin), "Admin");
    assert_eq!(format!("{}", UserRole::User), "User");
}

// ---------------------------------------------------------------------------
// TC-TYPE-03: LoginResponse deserialization
// ---------------------------------------------------------------------------

/// TC-TYPE-03: A well-formed JSON payload deserializes into `LoginResponse`
/// with correct `token`, `user.name`, and `user.role`.
#[wasm_bindgen_test]
fn test_login_response_deserialize() {
    let json = r#"{
        "token": "jwt-abc-123",
        "user": {
            "id": "user-001",
            "name": "田中太郎",
            "email": "tanaka@example.com",
            "role": "Admin",
            "created_at": "2024-01-01T00:00:00Z"
        }
    }"#;

    let resp: LoginResponse =
        serde_json::from_str(json).expect("LoginResponse should deserialize");

    assert_eq!(resp.token, "jwt-abc-123");
    assert_eq!(resp.user.name, "田中太郎");
    assert_eq!(resp.user.role, UserRole::Admin);
}

// ---------------------------------------------------------------------------
// TC-TYPE-04: Expense deserialization — note is null
// ---------------------------------------------------------------------------

/// TC-TYPE-04: When the JSON field `note` is `null`, the Rust field is `None`.
#[wasm_bindgen_test]
fn test_expense_deserialize_null_note() {
    let json = r#"{
        "id": "exp-001",
        "user_id": "user-001",
        "category_id": "cat-001",
        "amount": 1500,
        "purpose": "交通費精算",
        "occurred_at": "2024-03-01",
        "note": null,
        "receipt_file": "receipt.jpg",
        "status": "Pending",
        "created_at": "2024-03-01T09:00:00Z"
    }"#;

    let expense: Expense =
        serde_json::from_str(json).expect("Expense should deserialize");

    assert_eq!(
        expense.note, None,
        "note field should be None when JSON value is null"
    );
}

// ---------------------------------------------------------------------------
// TC-TYPE-05: Expense deserialization — receipt_file is null
// ---------------------------------------------------------------------------

/// TC-TYPE-05: When the JSON field `receipt_file` is `null`, the Rust field
/// is `None`.
#[wasm_bindgen_test]
fn test_expense_deserialize_null_receipt() {
    let json = r#"{
        "id": "exp-002",
        "user_id": "user-001",
        "category_id": "cat-001",
        "amount": 3000,
        "purpose": "宿泊費",
        "occurred_at": "2024-03-02",
        "note": "出張のため",
        "receipt_file": null,
        "status": "Approved",
        "created_at": "2024-03-02T09:00:00Z"
    }"#;

    let expense: Expense =
        serde_json::from_str(json).expect("Expense should deserialize");

    assert_eq!(
        expense.receipt_file, None,
        "receipt_file field should be None when JSON value is null"
    );
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

/// Verify that `Category` round-trips through serde correctly.
#[wasm_bindgen_test]
fn test_category_serde_roundtrip() {
    let cat = Category {
        id: "cat-001".to_string(),
        name: "交通費".to_string(),
    };
    let json = serde_json::to_string(&cat).expect("serialize");
    let restored: Category = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(cat, restored);
}

/// Verify `ExpenseStatus` serde uses PascalCase per the `rename_all` attribute.
#[wasm_bindgen_test]
fn test_expense_status_serde_pascal_case() {
    let json = r#""Pending""#;
    let status: ExpenseStatus =
        serde_json::from_str(json).expect("ExpenseStatus should deserialize from PascalCase");
    assert_eq!(status, ExpenseStatus::Pending);

    let serialized = serde_json::to_string(&ExpenseStatus::Rejected)
        .expect("serialize");
    assert_eq!(serialized, r#""Rejected""#);
}

/// Verify `UserRole` serde uses PascalCase per the `rename_all` attribute.
#[wasm_bindgen_test]
fn test_user_role_serde_pascal_case() {
    let json = r#""Admin""#;
    let role: UserRole =
        serde_json::from_str(json).expect("UserRole should deserialize from PascalCase");
    assert_eq!(role, UserRole::Admin);
}
