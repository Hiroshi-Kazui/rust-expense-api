// TC-STORE-01 through TC-STORE-09
// Tests for store.rs: AuthStore reactive signals and localStorage persistence.
// Must run in a browser environment because gloo-storage uses the Web Storage API.

use wasm_bindgen_test::wasm_bindgen_test;
use gloo_storage::{LocalStorage, Storage};
use frontend::{
    store::{AuthStore, TOKEN_KEY, USER_KEY},
    types::{User, UserRole},
};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helper: produces a deterministic User for each role.
// ---------------------------------------------------------------------------
fn mock_user(role: UserRole) -> User {
    User {
        id: "u-test-001".to_string(),
        name: "テストユーザー".to_string(),
        email: "test@example.com".to_string(),
        role,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

/// Clean up localStorage keys used by AuthStore so tests don't bleed into
/// each other.  Call at the start of each test that touches localStorage.
fn clear_auth_storage() {
    LocalStorage::delete(TOKEN_KEY);
    LocalStorage::delete(USER_KEY);
}

// ---------------------------------------------------------------------------
// TC-STORE-01: login() sets token and user signals
// ---------------------------------------------------------------------------

/// TC-STORE-01: After calling `login()`, the reactive signals hold the
/// supplied token and user values.
#[wasm_bindgen_test]
fn test_auth_store_login_sets_signals() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    owner.with(|| {
        let store = AuthStore::new();
        let user = mock_user(UserRole::User);

        store.login("test-token".to_string(), user.clone());

        assert_eq!(
            store.token.get(),
            Some("test-token".to_string()),
            "token signal should equal the value passed to login()"
        );
        assert_eq!(
            store.user.get(),
            Some(user),
            "user signal should equal the value passed to login()"
        );
    });
}

// ---------------------------------------------------------------------------
// TC-STORE-02: login() persists token to localStorage
// ---------------------------------------------------------------------------

/// TC-STORE-02: After `login()`, the token is readable from `LocalStorage`.
#[wasm_bindgen_test]
fn test_auth_store_login_saves_to_localstorage() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    owner.with(|| {
        let store = AuthStore::new();
        store.login("test-token".to_string(), mock_user(UserRole::User));
    });

    let stored: Result<String, _> = LocalStorage::get(TOKEN_KEY);
    assert_eq!(
        stored.expect("TOKEN_KEY should be present in localStorage after login"),
        "test-token"
    );

    clear_auth_storage();
}

// ---------------------------------------------------------------------------
// TC-STORE-03: logout() clears signals
// ---------------------------------------------------------------------------

/// TC-STORE-03: After `logout()`, both reactive signals are `None`.
#[wasm_bindgen_test]
fn test_auth_store_logout_clears_signals() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    owner.with(|| {
        let store = AuthStore::new();
        store.login("tok".to_string(), mock_user(UserRole::User));

        store.logout();

        assert_eq!(
            store.token.get(),
            None,
            "token signal should be None after logout()"
        );
        assert_eq!(
            store.user.get(),
            None,
            "user signal should be None after logout()"
        );
    });
}

// ---------------------------------------------------------------------------
// TC-STORE-04: logout() removes keys from localStorage
// ---------------------------------------------------------------------------

/// TC-STORE-04: After `logout()`, `LocalStorage::get(TOKEN_KEY)` returns `Err`.
#[wasm_bindgen_test]
fn test_auth_store_logout_removes_from_localstorage() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    owner.with(|| {
        let store = AuthStore::new();
        store.login("tok".to_string(), mock_user(UserRole::User));
        store.logout();
    });

    let stored: Result<String, _> = LocalStorage::get(TOKEN_KEY);
    assert!(
        stored.is_err(),
        "TOKEN_KEY should not exist in localStorage after logout()"
    );
}

// ---------------------------------------------------------------------------
// TC-STORE-05: is_authenticated() — token present
// ---------------------------------------------------------------------------

/// TC-STORE-05: `is_authenticated()` returns `true` when a token is set.
#[wasm_bindgen_test]
fn test_is_authenticated_returns_true_with_token() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    let result = owner.with(|| {
        let store = AuthStore::new();
        store.login("tok".to_string(), mock_user(UserRole::User));
        store.is_authenticated()
    });

    assert!(result, "is_authenticated() should return true after login");
}

// ---------------------------------------------------------------------------
// TC-STORE-06: is_authenticated() — no token
// ---------------------------------------------------------------------------

/// TC-STORE-06: `is_authenticated()` returns `false` when no token has been
/// set and localStorage is empty.
#[wasm_bindgen_test]
fn test_is_authenticated_returns_false_without_token() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    let result = owner.with(|| {
        let store = AuthStore::new();
        store.is_authenticated()
    });

    assert!(
        !result,
        "is_authenticated() should return false on a fresh AuthStore with empty localStorage"
    );
}

// ---------------------------------------------------------------------------
// TC-STORE-07: is_admin() — Admin role
// ---------------------------------------------------------------------------

/// TC-STORE-07: `is_admin()` returns `true` when the logged-in user has
/// `UserRole::Admin`.
#[wasm_bindgen_test]
fn test_is_admin_returns_true_for_admin() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    let result = owner.with(|| {
        let store = AuthStore::new();
        store.login("tok".to_string(), mock_user(UserRole::Admin));
        store.is_admin()
    });

    assert!(result, "is_admin() should return true for UserRole::Admin");
}

// ---------------------------------------------------------------------------
// TC-STORE-08: is_admin() — User role
// ---------------------------------------------------------------------------

/// TC-STORE-08: `is_admin()` returns `false` when the logged-in user has
/// `UserRole::User`.
#[wasm_bindgen_test]
fn test_is_admin_returns_false_for_user() {
    clear_auth_storage();

    let owner = leptos::reactive::owner::Owner::new();
    let result = owner.with(|| {
        let store = AuthStore::new();
        store.login("tok".to_string(), mock_user(UserRole::User));
        store.is_admin()
    });

    assert!(
        !result,
        "is_admin() should return false for UserRole::User"
    );
}

// ---------------------------------------------------------------------------
// TC-STORE-09: AuthStore::new() restores state from localStorage
// ---------------------------------------------------------------------------

/// TC-STORE-09: When a token was previously written to `LocalStorage`,
/// `AuthStore::new()` picks it up and the token signal is non-None.
#[wasm_bindgen_test]
fn test_auth_store_restores_from_localstorage() {
    clear_auth_storage();

    // Pre-populate localStorage as if a previous session had saved a token.
    LocalStorage::set(TOKEN_KEY, "saved-token")
        .expect("pre-populate TOKEN_KEY for TC-STORE-09");

    let owner = leptos::reactive::owner::Owner::new();
    let token_value = owner.with(|| {
        let store = AuthStore::new();
        store.token.get()
    });

    assert_eq!(
        token_value,
        Some("saved-token".to_string()),
        "AuthStore::new() should restore the token from localStorage"
    );

    clear_auth_storage();
}
