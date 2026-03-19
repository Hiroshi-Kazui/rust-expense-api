# フロントエンド テスト仕様書

**対象**: `frontend`（Leptos 0.7 CSR / WebAssembly）
**テストフレームワーク**:
- 単体テスト: `wasm-bindgen-test`（`wasm-pack test`）
- E2Eテスト: `playwright`（ブラウザ自動操作）
**実行環境**: Chrome（headless） / Firefox（headless）

---

## テスト方針

| 方針 | 内容 |
|---|---|
| 単体テスト | WASM上で動作する型変換・ストア・API関数をテスト |
| コンポーネントテスト | Leptos コンポーネントを `wasm-bindgen-test` で DOM レベルでテスト |
| E2Eテスト | バックエンド起動済みの環境でブラウザ操作をPlaywrightで自動化 |
| APIモック | コンポーネントテストでは `msw` または `fetch` モックを使用 |

---

## 1. 単体テスト（WASM）

### 1.1 型変換（`src/types.rs`）

#### TC-TYPE-01: `ExpenseStatus` の `Display` 実装

| 項目 | 内容 |
|---|---|
| テストID | TC-TYPE-01 |
| テスト名 | `test_expense_status_display` |
| 手順 | `format!("{}", ExpenseStatus::Pending)` 等を呼ぶ |
| 期待結果 | `Pending` → `"申請中"` / `Approved` → `"承認済"` / `Rejected` → `"差し戻し"` |

#### TC-TYPE-02: `UserRole` の `Display` 実装

| 項目 | 内容 |
|---|---|
| テストID | TC-TYPE-02 |
| テスト名 | `test_user_role_display` |
| 期待結果 | `Admin` → `"Admin"` / `User` → `"User"` |

#### TC-TYPE-03: `LoginResponse` のデシリアライズ

| 項目 | 内容 |
|---|---|
| テストID | TC-TYPE-03 |
| テスト名 | `test_login_response_deserialize` |
| 手順 | 正常なJSONをserde_json::from_str でデシリアライズ |
| 期待結果 | `token`・`user.name`・`user.role` が正しく復元される |

#### TC-TYPE-04: `Expense` のデシリアライズ — `note` が null の場合

| 項目 | 内容 |
|---|---|
| テストID | TC-TYPE-04 |
| テスト名 | `test_expense_deserialize_null_note` |
| 手順 | `"note": null` を含むJSONをデシリアライズ |
| 期待結果 | `note: None` |

#### TC-TYPE-05: `Expense` のデシリアライズ — `receipt_file` が null の場合

| 項目 | 内容 |
|---|---|
| テストID | TC-TYPE-05 |
| テスト名 | `test_expense_deserialize_null_receipt` |
| 期待結果 | `receipt_file: None` |

---

### 1.2 認証ストア（`src/store.rs`）

以下のテストは `wasm-bindgen-test` 環境（ブラウザ）で実行する。

#### TC-STORE-01: `login()` でトークンとユーザーがシグナルにセットされる

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-01 |
| テスト名 | `test_auth_store_login_sets_signals` |
| 前提条件 | 空の `AuthStore::new()` |
| 手順 | `store.login("test-token".into(), mock_user())` を呼ぶ |
| 期待結果 | `store.token.get() == Some("test-token")` / `store.user.get() == Some(mock_user())` |

#### TC-STORE-02: `login()` で localStorage に保存される

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-02 |
| テスト名 | `test_auth_store_login_saves_to_localstorage` |
| 手順 | `store.login(token, user)` 後に `LocalStorage::get::<String>(TOKEN_KEY)` を呼ぶ |
| 期待結果 | `Ok("test-token")` が返る |

#### TC-STORE-03: `logout()` でシグナルがクリアされる

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-03 |
| テスト名 | `test_auth_store_logout_clears_signals` |
| 前提条件 | login済みのストア |
| 手順 | `store.logout()` を呼ぶ |
| 期待結果 | `store.token.get() == None` / `store.user.get() == None` |

#### TC-STORE-04: `logout()` で localStorage が削除される

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-04 |
| テスト名 | `test_auth_store_logout_removes_from_localstorage` |
| 手順 | `logout()` 後に `LocalStorage::get::<String>(TOKEN_KEY)` を呼ぶ |
| 期待結果 | `Err(_)` が返る（キーが存在しない） |

#### TC-STORE-05: `is_authenticated()` — トークンあり

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-05 |
| テスト名 | `test_is_authenticated_returns_true_with_token` |
| 前提条件 | login済みのストア |
| 期待結果 | `true` |

#### TC-STORE-06: `is_authenticated()` — トークンなし

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-06 |
| テスト名 | `test_is_authenticated_returns_false_without_token` |
| 前提条件 | 未ログインのストア |
| 期待結果 | `false` |

#### TC-STORE-07: `is_admin()` — Adminロール

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-07 |
| テスト名 | `test_is_admin_returns_true_for_admin` |
| 前提条件 | `UserRole::Admin` のユーザーでlogin |
| 期待結果 | `true` |

#### TC-STORE-08: `is_admin()` — Userロール

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-08 |
| テスト名 | `test_is_admin_returns_false_for_user` |
| 前提条件 | `UserRole::User` のユーザーでlogin |
| 期待結果 | `false` |

#### TC-STORE-09: `AuthStore::new()` — localStorage から復元

| 項目 | 内容 |
|---|---|
| テストID | TC-STORE-09 |
| テスト名 | `test_auth_store_restores_from_localstorage` |
| 前提条件 | 事前に `LocalStorage::set(TOKEN_KEY, "saved-token")` しておく |
| 手順 | `AuthStore::new()` を呼ぶ |
| 期待結果 | `store.token.get() == Some("saved-token")` |

---

### 1.3 エラー型（`src/error.rs`）

#### TC-ERR-FE-01: `AppError::Display` の確認

| 項目 | 内容 |
|---|---|
| テストID | TC-ERR-FE-01 |
| テスト名 | `test_app_error_display` |
| 手順 | 各バリアントに `format!("{}", err)` を呼ぶ |
| 期待結果 | `Network("detail")` → `"ネットワークエラー: detail"` 等 |

---

## 2. コンポーネントテスト

### 前提: コンポーネントテストの構成

```rust
#[wasm_bindgen_test]
async fn test_login_form_renders() {
    // Leptos テストユーティリティでコンポーネントをマウント
    // DOM を検査して期待値を確認
}
```

---

### 2.1 ログイン画面（`src/pages/login.rs`）

#### TC-LOGIN-01: 初期レンダリング

| 項目 | 内容 |
|---|---|
| テストID | TC-LOGIN-01 |
| テスト名 | `test_login_page_renders` |
| 期待DOM | email入力フィールド、password入力フィールド、「ログイン」ボタンが存在する |

#### TC-LOGIN-02: 空フォーム送信でエラー表示

| 項目 | 内容 |
|---|---|
| テストID | TC-LOGIN-02 |
| テスト名 | `test_login_empty_form_shows_error` |
| 手順 | フィールドを空のまま「ログイン」ボタンをクリック |
| 期待DOM | `"メールアドレスとパスワードを入力してください"` のエラーテキストが表示される |

#### TC-LOGIN-03: APIエラー時にエラーメッセージ表示

| 項目 | 内容 |
|---|---|
| テストID | TC-LOGIN-03 |
| テスト名 | `test_login_api_error_shows_message` |
| 前提条件 | `POST /auth/login` が 401 を返すようモック |
| 手順 | 有効な形式で入力してログインボタンをクリック |
| 期待DOM | エラーメッセージが表示される |

#### TC-LOGIN-04: ローディング中はボタンが無効化

| 項目 | 内容 |
|---|---|
| テストID | TC-LOGIN-04 |
| テスト名 | `test_login_button_disabled_while_loading` |
| 手順 | APIレスポンス待ち中（Promise pending状態）にボタン状態を確認 |
| 期待DOM | ボタンが `disabled` かつテキストが `"ログイン中..."` |

---

### 2.2 経費申請一覧（`src/pages/expenses/list.rs`）

#### TC-LIST-01: 申請リストのレンダリング

| 項目 | 内容 |
|---|---|
| テストID | TC-LIST-01 |
| テスト名 | `test_expense_list_renders_items` |
| 前提条件 | `GET /expenses` が3件の申請を返すようモック |
| 期待DOM | テーブルに3行のデータが表示される |

#### TC-LIST-02: 空リスト時のメッセージ表示

| 項目 | 内容 |
|---|---|
| テストID | TC-LIST-02 |
| テスト名 | `test_expense_list_empty_state` |
| 前提条件 | `GET /expenses` が空配列を返すようモック |
| 期待DOM | `"申請がありません"` のメッセージが表示される |

#### TC-LIST-03: Pending申請にのみ「編集」リンクが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-LIST-03 |
| テスト名 | `test_expense_list_edit_link_only_for_pending` |
| 前提条件 | Pending/Approved/Rejected 各1件のデータをモック |
| 期待DOM | Pending行のみ `href="/expenses/{id}/edit"` のリンクが存在 |

#### TC-LIST-04: ステータスバッジのクラスが正しい

| 項目 | 内容 |
|---|---|
| テストID | TC-LIST-04 |
| テスト名 | `test_expense_list_status_badge_classes` |
| 期待DOM | Pending → `badge-pending` クラス / Approved → `badge-approved` / Rejected → `badge-rejected` |

#### TC-LIST-05: 「新規申請」ボタンが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-LIST-05 |
| テスト名 | `test_expense_list_new_button_exists` |
| 期待DOM | `href="/expenses/new"` のリンクが存在する |

---

### 2.3 経費申請作成フォーム（`src/pages/expenses/create.rs`）

#### TC-CREATE-01: カテゴリ選択肢の表示

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-01 |
| テスト名 | `test_expense_create_category_options` |
| 前提条件 | `GET /categories` が3件のカテゴリを返すようモック |
| 期待DOM | セレクトボックスに3件のオプションが表示される |

#### TC-CREATE-02: 必須フィールド未入力でエラー表示

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-02 |
| テスト名 | `test_expense_create_required_field_validation` |
| 手順 | 目的・金額・発生日を空にしてフォーム送信 |
| 期待DOM | `"必須項目を入力してください"` のエラーが表示される |

#### TC-CREATE-03: 金額に非数値を入力するとエラー

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-03 |
| テスト名 | `test_expense_create_invalid_amount` |
| 手順 | 金額に `"abc"` を入力して送信 |
| 期待DOM | `"金額は正の整数で入力してください"` |

#### TC-CREATE-04: 金額に0を入力するとエラー

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-04 |
| テスト名 | `test_expense_create_zero_amount` |
| 手順 | 金額に `0` を入力して送信 |
| 期待DOM | バリデーションエラーが表示される |

#### TC-CREATE-05: ファイルアップロードフィールドの表示

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-05 |
| テスト名 | `test_expense_create_file_input_exists` |
| 期待DOM | `input[type=file][accept=".jpg,.jpeg,.png,.pdf"]` が存在する |

#### TC-CREATE-06: 送信中はボタンが無効化

| 項目 | 内容 |
|---|---|
| テストID | TC-CREATE-06 |
| テスト名 | `test_expense_create_submit_button_disabled_while_loading` |
| 期待DOM | API待ち中にボタンが `disabled` かつ `"送信中..."` |

---

### 2.4 経費申請編集フォーム（`src/pages/expenses/edit.rs`）

#### TC-EDIT-01: 既存データがフォームに初期表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-EDIT-01 |
| テスト名 | `test_expense_edit_prefills_form` |
| 前提条件 | `GET /expenses/{id}` が既存データを返すようモック |
| 期待DOM | 金額・目的・発生日フィールドが既存値で埋まっている |

#### TC-EDIT-02: 削除ボタン — 1回目クリックで確認表示

| 項目 | 内容 |
|---|---|
| テストID | TC-EDIT-02 |
| テスト名 | `test_expense_edit_delete_confirm_on_first_click` |
| 手順 | 「削除」ボタンを1回クリック |
| 期待DOM | ボタンテキストが `"本当に削除"` に変わる |

#### TC-EDIT-03: 削除ボタン — 2回目クリックで削除API呼び出し

| 項目 | 内容 |
|---|---|
| テストID | TC-EDIT-03 |
| テスト名 | `test_expense_edit_delete_on_second_click` |
| 前提条件 | `DELETE /expenses/{id}` が204を返すようモック |
| 手順 | 削除ボタンを2回クリック |
| 期待結果 | `DELETE /expenses/{id}` が1回呼ばれる |

---

### 2.5 承認管理画面（`src/pages/admin_expenses.rs`）

#### TC-ADMIN-01: 全申請一覧のレンダリング

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-01 |
| テスト名 | `test_admin_expenses_renders_all` |
| 前提条件 | `GET /expenses` が複数ユーザーの申請を返すようモック |
| 期待DOM | 全件がテーブルに表示される |

#### TC-ADMIN-02: Pending申請のみチェックボックスが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-02 |
| テスト名 | `test_admin_expenses_checkbox_only_for_pending` |
| 前提条件 | Pending/Approved/Rejected 各1件 |
| 期待DOM | Pending行のみ `input[type=checkbox]` が存在する |

#### TC-ADMIN-03: チェックボックス選択数が一括承認ボタンに反映

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-03 |
| テスト名 | `test_admin_expenses_bulk_button_shows_selected_count` |
| 手順 | Pending申請を2件チェック |
| 期待DOM | ボタンテキストが `"一括承認（2件選択中）"` になる |

#### TC-ADMIN-04: Pending申請に承認/差し戻しボタンが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-04 |
| テスト名 | `test_admin_expenses_action_buttons_for_pending` |
| 期待DOM | Pending行に `"承認"` と `"差し戻し"` ボタンが存在する |

#### TC-ADMIN-05: 非Pending申請は「戻す」ボタンのみ

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-05 |
| テスト名 | `test_admin_expenses_revert_button_for_non_pending` |
| 期待DOM | Approved/Rejected行には `"戻す"` ボタンのみ存在する |

#### TC-ADMIN-06: 一括承認ボタン — 未選択時はエラー表示

| 項目 | 内容 |
|---|---|
| テストID | TC-ADMIN-06 |
| テスト名 | `test_admin_expenses_bulk_approve_without_selection` |
| 手順 | 何も選択せずに「一括承認」ボタンをクリック |
| 期待DOM | `"申請を選択してください"` エラーが表示される |

---

### 2.6 サイドバーレイアウト（`src/components/layout.rs`）

#### TC-LAYOUT-01: Adminユーザーに管理メニューが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-LAYOUT-01 |
| テスト名 | `test_layout_admin_menu_visible_for_admin` |
| 前提条件 | `UserRole::Admin` で AuthStore に provide_context |
| 期待DOM | `href="/admin/expenses"` `href="/admin/users"` `href="/admin/categories"` のリンクが存在する |

#### TC-LAYOUT-02: 一般ユーザーに管理メニューが表示されない

| 項目 | 内容 |
|---|---|
| テストID | TC-LAYOUT-02 |
| テスト名 | `test_layout_admin_menu_hidden_for_user` |
| 前提条件 | `UserRole::User` で AuthStore に provide_context |
| 期待DOM | `href="/admin/expenses"` リンクが存在しない |

#### TC-LAYOUT-03: ユーザー名がサイドバーに表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-LAYOUT-03 |
| テスト名 | `test_layout_shows_user_name` |
| 前提条件 | `name: "田中太郎"` のユーザーでログイン |
| 期待DOM | `"田中太郎"` が表示される |

---

### 2.7 トースト通知（`src/components/toast.rs`）

#### TC-TOAST-01: Success メッセージが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-TOAST-01 |
| テスト名 | `test_toast_shows_success_message` |
| 手順 | `show_toast("保存しました".into(), ToastKind::Success)` を呼ぶ |
| 期待DOM | `"保存しました"` テキストを持つ要素が表示される、`bg-green-500` クラスが付く |

#### TC-TOAST-02: Error メッセージが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-TOAST-02 |
| テスト名 | `test_toast_shows_error_message` |
| 手順 | `show_toast("エラーが発生しました".into(), ToastKind::Error)` を呼ぶ |
| 期待DOM | `bg-red-500` クラスが付く |

#### TC-TOAST-03: 3秒後に自動消去される

| 項目 | 内容 |
|---|---|
| テストID | TC-TOAST-03 |
| テスト名 | `test_toast_auto_dismiss_after_3s` |
| 手順 | トーストを表示し、3秒後の状態を確認 |
| 期待DOM | 3秒後にトーストが DOM から削除される |

---

## 3. E2Eテスト（Playwright）

### 前提条件

- `docker compose up --build` でサービス起動済み
- フロントエンド: `http://localhost:3000`
- バックエンドAPI: `http://localhost:8080`
- 初期データ: adminユーザー (`admin@example.com` / `changeme`) 作成済み

---

### 3.1 認証フロー

#### TC-E2E-AUTH-01: ログイン成功 → ダッシュボードにリダイレクト

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-AUTH-01 |
| テスト名 | `e2e_login_success_redirects_to_expenses` |
| 手順 | 1. `http://localhost:3000/login` にアクセス<br>2. email・password を入力<br>3. 「ログイン」ボタンをクリック |
| 期待動作 | URL が `/expenses` に遷移する |
| 期待DOM | 「経費申請一覧」見出しが表示される |

#### TC-E2E-AUTH-02: 誤パスワードでエラーメッセージ表示

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-AUTH-02 |
| テスト名 | `e2e_login_wrong_password_shows_error` |
| 手順 | 誤ったパスワードを入力してログインボタンをクリック |
| 期待DOM | エラーメッセージが表示される、URLは `/login` のまま |

#### TC-E2E-AUTH-03: 未認証でのページアクセス → ログイン画面にリダイレクト

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-AUTH-03 |
| テスト名 | `e2e_unauthenticated_redirects_to_login` |
| 手順 | localStorage を空にして `/expenses` に直接アクセス |
| 期待動作 | `/login` にリダイレクトされる |

#### TC-E2E-AUTH-04: ログアウト → ログイン画面にリダイレクト

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-AUTH-04 |
| テスト名 | `e2e_logout_redirects_to_login` |
| 手順 | ログイン後、サイドバーの「ログアウト」をクリック |
| 期待動作 | `/login` にリダイレクト、localStorage からトークンが削除される |

---

### 3.2 経費申請フロー（一般ユーザー）

#### TC-E2E-EXP-01: 経費申請の作成

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-EXP-01 |
| テスト名 | `e2e_create_expense_success` |
| 前提条件 | 一般ユーザーでログイン済み、勘定項目（交通費）が存在 |
| 手順 | 1. サイドバーの「新規申請」クリック<br>2. 各フィールドを入力<br>3. 「申請する」クリック |
| 期待動作 | `/expenses` にリダイレクト |
| 期待DOM | 一覧に新しい申請が「申請中」バッジで表示される |

#### TC-E2E-EXP-02: 経費申請の作成（ファイル添付）

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-EXP-02 |
| テスト名 | `e2e_create_expense_with_receipt` |
| 手順 | ファイル入力フィールドにJPEGを設定して申請する |
| 期待動作 | 申請作成成功、一覧に戻る |

#### TC-E2E-EXP-03: Pending申請の編集

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-EXP-03 |
| テスト名 | `e2e_edit_pending_expense` |
| 手順 | 1. 申請一覧で「編集」リンクをクリック<br>2. 金額を変更<br>3. 「更新する」クリック |
| 期待動作 | `/expenses` にリダイレクト、更新後の金額が一覧に表示される |

#### TC-E2E-EXP-04: Pending申請の削除

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-EXP-04 |
| テスト名 | `e2e_delete_pending_expense` |
| 手順 | 1. 編集画面で「削除」ボタンをクリック（1回目）<br>2. ボタンが「本当に削除」に変わることを確認<br>3. 再度クリック（2回目） |
| 期待動作 | `/expenses` にリダイレクト、削除した申請が一覧から消える |

#### TC-E2E-EXP-05: 他人の申請は一覧に表示されない

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-EXP-05 |
| テスト名 | `e2e_user_sees_own_expenses_only` |
| 前提条件 | ユーザーAとユーザーBがそれぞれ申請を作成済み |
| 手順 | ユーザーAでログインして申請一覧を確認 |
| 期待DOM | ユーザーBの申請が表示されない |

---

### 3.3 管理者フロー（Admin）

#### TC-E2E-ADMIN-01: 管理メニューが表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-01 |
| テスト名 | `e2e_admin_sees_admin_menu` |
| 前提条件 | admin@example.com でログイン |
| 期待DOM | サイドバーに「全申請一覧」「ユーザー」「勘定項目」リンクが表示される |

#### TC-E2E-ADMIN-02: 全申請一覧に全ユーザーの申請が表示される

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-02 |
| テスト名 | `e2e_admin_sees_all_expenses` |
| 前提条件 | 複数ユーザーの申請が存在 |
| 手順 | 「全申請一覧」をクリック |
| 期待DOM | 複数ユーザーの申請が全てテーブルに表示される |

#### TC-E2E-ADMIN-03: 個別承認

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-03 |
| テスト名 | `e2e_admin_approve_single_expense` |
| 手順 | 1. 全申請一覧でPending申請の「承認」ボタンをクリック |
| 期待DOM | バッジが「承認済」に変わる |

#### TC-E2E-ADMIN-04: 個別差し戻し

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-04 |
| テスト名 | `e2e_admin_reject_single_expense` |
| 手順 | Pending申請の「差し戻し」ボタンをクリック |
| 期待DOM | バッジが「差し戻し」に変わる |

#### TC-E2E-ADMIN-05: 一括承認

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-05 |
| テスト名 | `e2e_admin_bulk_approve` |
| 手順 | 1. Pending申請を複数チェック<br>2. 「一括承認」ボタンをクリック |
| 期待DOM | 選択した申請すべてのバッジが「承認済」に変わる |

#### TC-E2E-ADMIN-06: ユーザー登録

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-06 |
| テスト名 | `e2e_admin_create_user` |
| 手順 | 1. 「ユーザー」メニューをクリック<br>2. フォームに入力して「登録」クリック |
| 期待DOM | 一覧に新しいユーザーが表示される |

#### TC-E2E-ADMIN-07: 勘定項目追加

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-07 |
| テスト名 | `e2e_admin_create_category` |
| 手順 | 1. 「勘定項目」メニューをクリック<br>2. 名前を入力して「追加」クリック |
| 期待DOM | 一覧に新しい勘定項目が表示される |

#### TC-E2E-ADMIN-08: 非Adminが管理ページにアクセスすると弾かれる

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-ADMIN-08 |
| テスト名 | `e2e_non_admin_cannot_access_admin_pages` |
| 手順 | 一般ユーザーで `/admin/users` に直接アクセス |
| 期待動作 | `/expenses` にリダイレクトされる |

---

### 3.4 フォームバリデーション（E2E）

#### TC-E2E-VAL-01: ログインフォーム — 空送信

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-VAL-01 |
| テスト名 | `e2e_login_form_empty_validation` |
| 手順 | 入力なしでログインボタンをクリック |
| 期待DOM | バリデーションエラーが表示される |

#### TC-E2E-VAL-02: 申請作成フォーム — 必須項目未入力

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-VAL-02 |
| テスト名 | `e2e_expense_create_required_validation` |
| 手順 | フィールドを空のまま「申請する」をクリック |
| 期待DOM | `"必須項目を入力してください"` が表示される |

#### TC-E2E-VAL-03: 申請作成 — 負の金額

| 項目 | 内容 |
|---|---|
| テストID | TC-E2E-VAL-03 |
| テスト名 | `e2e_expense_create_negative_amount_validation` |
| 手順 | 金額に `-100` を入力して送信 |
| 期待DOM | バリデーションエラーが表示される |

---

## 4. テストカバレッジ目標

| 対象 | 目標 |
|---|---|
| `types.rs`・`error.rs` | 100% |
| `store.rs` | 95%以上 |
| ページコンポーネント（主要フロー） | 80%以上 |
| E2Eシナリオ | 主要ユーザージャーニー全件カバー |

---

## 5. テスト実行方法

```bash
# WASM単体テスト（wasm-pack 要）
cd frontend
wasm-pack test --headless --chrome

# E2Eテスト（Playwrightを使用）
# バックエンド起動後:
cd e2e
npx playwright test

# 特定テストのみ実行
npx playwright test --grep "TC-E2E-AUTH"

# レポート確認
npx playwright show-report
```

---

## 6. テストデータ定義（E2E用）

| 変数 | 値 |
|---|---|
| `ADMIN_EMAIL` | `admin@example.com` |
| `ADMIN_PASSWORD` | `changeme` |
| `USER_EMAIL` | `test-user@example.com` |
| `USER_PASSWORD` | `password123` |
| `CATEGORY_NAME` | `交通費` |
| `FRONTEND_URL` | `http://localhost:3000` |

### E2Eテスト用 `playwright.config.ts` 抜粋

```typescript
export default defineConfig({
  testDir: './tests',
  baseURL: 'http://localhost:3000',
  use: {
    headless: true,
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox',  use: { ...devices['Desktop Firefox'] } },
  ],
});
```
