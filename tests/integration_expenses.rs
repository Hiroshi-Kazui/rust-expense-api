// TC-EXP-01〜18: Expense CRUD integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use rust_expense_api::models::expense::ExpenseStatus;
use serial_test::serial;
use serde_json::{json, Value};
use uuid::Uuid;

fn multipart_body(fields: &[(&str, &str)]) -> (String, Vec<u8>) {
    let boundary = "----TestBoundary12345";
    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n", name, value)
                .as_bytes(),
        );
    }
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    (format!("multipart/form-data; boundary={}", boundary), body)
}

fn multipart_body_with_file(
    fields: &[(&str, &str)],
    file_name: &str,
    file_content_type: &str,
    file_bytes: &[u8],
) -> (String, Vec<u8>) {
    let boundary = "----TestBoundary12345";
    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n", name, value)
                .as_bytes(),
        );
    }
    // file field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"receipt\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
            file_name, file_content_type
        )
        .as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    (format!("multipart/form-data; boundary={}", boundary), body)
}

/// TC-EXP-01: 経費申請作成 — 正常系（ファイルなし）
#[actix_rt::test]
#[serial]
async fn test_create_expense_success_without_file() {
    common::cleanup_db();
    let (user, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    let (content_type, body) = multipart_body(&[
        ("category_id", &category.id.to_string()),
        ("amount", "5000"),
        ("purpose", "交通費申請"),
        ("occurred_at", "2026-01-15"),
    ]);

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["amount"], 5000);
    assert_eq!(body["purpose"], "交通費申請");
    assert_eq!(body["status"], "Pending");
    assert!(body["receipt_file"].is_null());
    assert_eq!(body["user_id"], user.id.to_string());
}

/// TC-EXP-02: 経費申請作成 — JPEGファイルアップロード付き
#[actix_rt::test]
#[serial]
async fn test_create_expense_with_jpeg() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    let jpeg = common::minimal_jpeg();
    let (content_type, body) = multipart_body_with_file(
        &[
            ("category_id", &category.id.to_string()),
            ("amount", "3000"),
            ("purpose", "領収書テスト"),
            ("occurred_at", "2026-01-20"),
        ],
        "receipt.jpg",
        "image/jpeg",
        &jpeg,
    );

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    let receipt = body["receipt_file"].as_str().unwrap();
    assert!(receipt.ends_with(".jpg"));

    // Cleanup uploaded file
    let _ = std::fs::remove_file(format!("./test_uploads/{}", receipt));
}

/// TC-EXP-03: 経費申請作成 — PDFファイルアップロード付き
#[actix_rt::test]
#[serial]
async fn test_create_expense_with_pdf() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    // Minimal PDF header
    let pdf_bytes = b"%PDF-1.4\n%%EOF\n";
    let (content_type, body) = multipart_body_with_file(
        &[
            ("category_id", &category.id.to_string()),
            ("amount", "2000"),
            ("purpose", "PDF領収書"),
            ("occurred_at", "2026-01-20"),
        ],
        "receipt.pdf",
        "application/pdf",
        pdf_bytes,
    );

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    let receipt = body["receipt_file"].as_str().unwrap();
    assert!(receipt.ends_with(".pdf"));
    let _ = std::fs::remove_file(format!("./test_uploads/{}", receipt));
}

/// TC-EXP-04: 経費申請作成 — 不正ファイル形式
#[actix_rt::test]
#[serial]
async fn test_create_expense_invalid_file_type() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    let (content_type, body) = multipart_body_with_file(
        &[
            ("category_id", &category.id.to_string()),
            ("amount", "1000"),
            ("purpose", "テスト"),
            ("occurred_at", "2026-01-15"),
        ],
        "receipt.txt",
        "text/plain",
        b"hello",
    );

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Unsupported file type"));
}

/// TC-EXP-06: 経費申請作成 — 金額0以下
#[actix_rt::test]
#[serial]
async fn test_create_expense_zero_amount() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    let (content_type, body) = multipart_body(&[
        ("category_id", &category.id.to_string()),
        ("amount", "0"),
        ("purpose", "テスト"),
        ("occurred_at", "2026-01-15"),
    ]);

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

/// TC-EXP-07a: 経費申請作成 — category_idなし
#[actix_rt::test]
#[serial]
async fn test_create_expense_missing_category_id() {
    common::cleanup_db();
    let (_, user_token) = common::create_regular_user();
    let app = common::build_app().await;

    let (content_type, body) = multipart_body(&[
        ("amount", "1000"),
        ("purpose", "テスト"),
        ("occurred_at", "2026-01-15"),
    ]);

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

/// TC-EXP-08: 経費申請作成 — 認証なし
#[actix_rt::test]
#[serial]
async fn test_create_expense_no_auth() {
    common::cleanup_db();
    let category = common::create_category("交通費");
    let app = common::build_app().await;

    let (content_type, body) = multipart_body(&[
        ("category_id", &category.id.to_string()),
        ("amount", "1000"),
        ("purpose", "テスト"),
        ("occurred_at", "2026-01-15"),
    ]);

    let req = test::TestRequest::post()
        .uri("/expenses")
        .insert_header(("Content-Type", content_type))
        .set_payload(body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// TC-EXP-09: 経費申請一覧 — User は自分のものだけ
#[actix_rt::test]
#[serial]
async fn test_list_expenses_user_sees_own_only() {
    common::cleanup_db();
    let (user_a, token_a) = common::create_regular_user();
    let (user_b, _) = common::create_regular_user();
    let category = common::create_category("交通費");

    common::create_expense(user_a.id, category.id, ExpenseStatus::Pending);
    common::create_expense(user_b.id, category.id, ExpenseStatus::Pending);

    let app = common::build_app().await;
    let req = test::TestRequest::get()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    let expenses = body.as_array().unwrap();
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0]["user_id"], user_a.id.to_string());
}

/// TC-EXP-10: 経費申請一覧 — Admin は全員分
#[actix_rt::test]
#[serial]
async fn test_list_expenses_admin_sees_all() {
    common::cleanup_db();
    let (_admin, admin_token) = common::create_admin();
    let (user_a, _) = common::create_regular_user();
    let (user_b, _) = common::create_regular_user();
    let category = common::create_category("交通費");

    common::create_expense(user_a.id, category.id, ExpenseStatus::Pending);
    common::create_expense(user_b.id, category.id, ExpenseStatus::Pending);

    let app = common::build_app().await;
    let req = test::TestRequest::get()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().len() >= 2);
}

/// TC-EXP-11: 経費申請一覧 — 作成日時降順
#[actix_rt::test]
#[serial]
async fn test_list_expenses_ordered_by_created_at_desc() {
    common::cleanup_db();
    let (user, token) = common::create_regular_user();
    let category = common::create_category("交通費");

    // Create 3 expenses in sequence
    let e1 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let e2 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let e3 = common::create_expense(user.id, category.id, ExpenseStatus::Pending);

    let app = common::build_app().await;
    let req = test::TestRequest::get()
        .uri("/expenses")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    let ids: Vec<&str> = body.as_array().unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    // Most recent first
    assert_eq!(ids[0], e3.id.to_string());
    assert_eq!(ids[ids.len() - 1], e1.id.to_string());
}

/// TC-EXP-12: 経費申請更新 — 正常系（Pending）
#[actix_rt::test]
#[serial]
async fn test_update_expense_success() {
    common::cleanup_db();
    let (user, token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"amount": 8000, "purpose": "新幹線代"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["amount"], 8000);
    assert_eq!(body["purpose"], "新幹線代");
}

/// TC-EXP-13: 経費申請更新 — Approved 申請は編集不可
#[actix_rt::test]
#[serial]
async fn test_update_expense_approved_fails() {
    common::cleanup_db();
    let (user, token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Approved);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"amount": 9000}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Only pending expenses can be edited"));
}

/// TC-EXP-14: 経費申請更新 — 他人の申請は編集不可
#[actix_rt::test]
#[serial]
async fn test_update_expense_other_user_fails() {
    common::cleanup_db();
    let (user_a, _) = common::create_regular_user();
    let (_, token_b) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user_a.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token_b)))
        .set_json(json!({"amount": 9000}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

/// TC-EXP-15: 経費申請更新 — 存在しないID
#[actix_rt::test]
#[serial]
async fn test_update_expense_not_found() {
    common::cleanup_db();
    let (_, token) = common::create_regular_user();
    let app = common::build_app().await;

    let req = test::TestRequest::patch()
        .uri(&format!("/expenses/{}", Uuid::new_v4()))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"amount": 1000}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

/// TC-EXP-16: 経費申請削除 — 正常系（Pending）
#[actix_rt::test]
#[serial]
async fn test_delete_expense_success() {
    common::cleanup_db();
    let (user, token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::delete()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);
}

/// TC-EXP-17: 経費申請削除 — Approved 申請は削除不可
#[actix_rt::test]
#[serial]
async fn test_delete_expense_approved_fails() {
    common::cleanup_db();
    let (user, token) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user.id, category.id, ExpenseStatus::Approved);
    let app = common::build_app().await;

    let req = test::TestRequest::delete()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

/// TC-EXP-18: 経費申請削除 — 他人の申請は削除不可
#[actix_rt::test]
#[serial]
async fn test_delete_expense_other_user_fails() {
    common::cleanup_db();
    let (user_a, _) = common::create_regular_user();
    let (_, token_b) = common::create_regular_user();
    let category = common::create_category("交通費");
    let expense = common::create_expense(user_a.id, category.id, ExpenseStatus::Pending);
    let app = common::build_app().await;

    let req = test::TestRequest::delete()
        .uri(&format!("/expenses/{}", expense.id))
        .insert_header(("Authorization", format!("Bearer {}", token_b)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}
