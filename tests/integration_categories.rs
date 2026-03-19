// TC-CAT-01〜06: Category management integration tests

mod common;

use actix_web::{dev::ServiceResponse, test};
use serial_test::serial;
use serde_json::{json, Value};

/// TC-CAT-01: 勘定項目作成 — 正常系
#[actix_rt::test]
#[serial]
async fn test_create_category_success() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/categories")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"name": "交通費"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "交通費");
    assert!(body["id"].is_string());
}

/// TC-CAT-02: 勘定項目作成 — 空白名
#[actix_rt::test]
#[serial]
async fn test_create_category_empty_name() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/categories")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"name": "   "}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Category name cannot be empty"));
}

/// TC-CAT-03: 勘定項目作成 — 名前がトリムされる
#[actix_rt::test]
#[serial]
async fn test_create_category_trims_whitespace() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/categories")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({"name": "  交通費  "}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "交通費");
}

/// TC-CAT-04: 勘定項目作成 — 認証なし
#[actix_rt::test]
#[serial]
async fn test_create_category_no_auth() {
    common::cleanup_db();
    let app = common::build_app().await;

    let req = test::TestRequest::post()
        .uri("/categories")
        .set_json(json!({"name": "交通費"}))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// TC-CAT-05: 勘定項目一覧取得 — 名前昇順
#[actix_rt::test]
#[serial]
async fn test_list_categories_sorted_by_name() {
    common::cleanup_db();
    let (_admin, token) = common::create_admin();
    // Insert in non-alphabetical order
    common::create_category("宿泊費");
    common::create_category("交通費");
    common::create_category("接待費");
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/categories")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: Value = test::read_body_json(resp).await;
    let names: Vec<&str> = body.as_array().unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

/// TC-CAT-06: 勘定項目一覧 — 非Admin
#[actix_rt::test]
#[serial]
async fn test_list_categories_as_non_admin() {
    common::cleanup_db();
    let (_user, token) = common::create_regular_user();
    let app = common::build_app().await;

    let req = test::TestRequest::get()
        .uri("/categories")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}
