// TC-APR-01〜11: Approval flow integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use rust_expense_api::models::expense::ExpenseStatus;
use serial_test::serial;
use serde_json::{json, Value};
use uuid::Uuid;

/// TC-APR-01: 個別承認 — Pending → Approved
#[actix_rt::test]
#[serial]
async fn test_update_status_pending_to_approved() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"status": "Approved"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Approved");
}

/// TC-APR-02: 個別差し戻し — Pending → Rejected
#[actix_rt::test]
#[serial]
async fn test_update_status_pending_to_rejected() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"status": "Rejected"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Rejected");
}

/// TC-APR-03: ステータス巻き戻し — Approved → Pending
#[actix_rt::test]
#[serial]
async fn test_update_status_revert_to_pending() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Approved);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"status": "Pending"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Pending");
}

/// TC-APR-04: ステータス更新 — 非Adminは操作不可
#[actix_rt::test]
#[serial]
async fn test_update_status_as_non_admin() {
    common::cleanup_db();
    let (user, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(json!({"status": "Approved"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

/// TC-APR-05: ステータス更新 — 不正なステータス値
#[actix_rt::test]
#[serial]
async fn test_update_status_invalid_value() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"status": "Unknown"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

/// TC-APR-06: ステータス更新 — 存在しないID
#[actix_rt::test]
#[serial]
async fn test_update_status_not_found() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}/status", Uuid::new_v4()))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"status": "Approved"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

/// TC-APR-07: 一括承認 — 正常系（Pending複数件）
#[actix_rt::test]
#[serial]
async fn test_bulk_approve_success() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");

    let e1 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let e2 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let e3 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);

    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/expenses/bulk-approve")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"ids": [e1.id, e2.id, e3.id]}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["approved"], 3);
}

/// TC-APR-08: 一括承認 — Pending以外は対象外
#[actix_rt::test]
#[serial]
async fn test_bulk_approve_skips_non_pending() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let (user, _) = common::create_regular_user();
    let category = common::create_category("交通費");

    let pending = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let approved = common::create_expense(user.id, category.id, ExpenseStatus::Approved);
    let rejected = common::create_expense(user.id, category.id, ExpenseStatus::Rejected);

    let app = common::build_app().await;
    let req = test::TestRequest::post()
        .uri("/expenses/bulk-approve")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"ids": [pending.id, approved.id, rejected.id]}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["approved"], 1);
}

/// TC-APR-09: 一括承認 — IDリスト空
#[actix_rt::test]
#[serial]
async fn test_bulk_approve_empty_ids() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/expenses/bulk-approve")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"ids": []}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("ids must not be empty"));
}

/// TC-APR-10: 一括承認 — 非Admin
#[actix_rt::test]
#[serial]
async fn test_bulk_approve_as_non_admin() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/expenses/bulk-approve")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(json!({"ids": [Uuid::new_v4()]}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

/// TC-APR-11: 一括承認 — 存在しないIDは無視される
#[actix_rt::test]
#[serial]
async fn test_bulk_approve_ignores_nonexistent_ids() {
    common::cleanup_db();
    let (_, admin_token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/expenses/bulk-approve")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({"ids": [Uuid::new_v4(), Uuid::new_v4()]}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["approved"], 0);
}
