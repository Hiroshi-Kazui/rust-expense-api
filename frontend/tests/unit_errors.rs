// TC-ERR-FE-01
// Tests for error.rs: AppError Display implementation.
// Pure Rust logic — no Web APIs required, but compiled for wasm32 target.

use wasm_bindgen_test::wasm_bindgen_test;
use frontend::error::AppError;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// TC-ERR-FE-01: AppError Display for all variants
// ---------------------------------------------------------------------------

/// TC-ERR-FE-01: Each `AppError` variant formats to the correct Japanese
/// prefix followed by the inner detail string.
#[wasm_bindgen_test]
fn test_app_error_display() {
    let cases: &[(&str, AppError)] = &[
        (
            "ネットワークエラー: connection refused",
            AppError::Network("connection refused".to_string()),
        ),
        (
            "認証エラー: token expired",
            AppError::Auth("token expired".to_string()),
        ),
        (
            "サーバーエラー: internal server error",
            AppError::Server("internal server error".to_string()),
        ),
        (
            "不正なリクエスト: invalid parameter",
            AppError::BadRequest("invalid parameter".to_string()),
        ),
    ];

    for (expected, error) in cases {
        let rendered = format!("{}", error);
        assert_eq!(
            &rendered, expected,
            "AppError::{:?} Display mismatch",
            error
        );
    }
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

/// An empty detail string should still produce the prefix with a trailing
/// colon-space, since the format is `"<prefix>: {msg}"`.
#[wasm_bindgen_test]
fn test_app_error_display_empty_message() {
    let error = AppError::Network("".to_string());
    assert_eq!(format!("{}", error), "ネットワークエラー: ");
}

/// `AppError` implements `Clone` — cloned value displays identically.
#[wasm_bindgen_test]
fn test_app_error_clone_displays_same() {
    let original = AppError::Auth("詳細メッセージ".to_string());
    let cloned = original.clone();
    assert_eq!(format!("{}", original), format!("{}", cloned));
}
