# バックエンド テスト仕様書

**対象**: `rust-expense-api`（actix-web + Diesel + PostgreSQL）
**テストフレームワーク**: Rust 標準テスト + `actix-web::test` + `diesel` テスト用DB
**テスト分類**: 単体テスト（Unit） / 統合テスト（Integration）

---

## テスト方針

| 方針 | 内容 |
|---|---|
| DBテスト | モックを使わず実際のPostgreSQLを使用する（テスト用DB） |
| テスト分離 | 各テストはトランザクションでラップし、終了後にロールバック |
| 認証テスト | JWTの生成・検証は単体テストで行う |
| HTTPテスト | `actix_web::test::init_service` を使ってHTTPレイヤーごとテスト |

---

## 1. 単体テスト

### 1.1 JWT モジュール（`src/auth/jwt.rs`）

#### TC-JWT-01: トークン生成 — 正常系

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-01 |
| テスト名 | `test_generate_token_success` |
| 前提条件 | 有効なユーザーID（UUID）、ロール（User）、シークレット、有効期限（24h） |
| 手順 | `generate_token(user_id, UserRole::User, "secret", 24)` を呼ぶ |
| 期待結果 | `Ok(String)` が返る。文字列は `.` で区切られた3セクション（JWT形式） |

#### TC-JWT-02: トークン検証 — 正常系

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-02 |
| テスト名 | `test_verify_token_success` |
| 前提条件 | TC-JWT-01 で生成したトークン |
| 手順 | `verify_token(token, "secret")` を呼ぶ |
| 期待結果 | `Ok(Claims)` が返る。`claims.sub` が元の `user_id` と一致する |

#### TC-JWT-03: トークン検証 — 署名不正

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-03 |
| テスト名 | `test_verify_token_wrong_secret` |
| 前提条件 | `"secret"` で生成したトークン |
| 手順 | `verify_token(token, "wrong-secret")` を呼ぶ |
| 期待結果 | `Err(AppError::Unauthorized(_))` が返る |

#### TC-JWT-04: トークン検証 — 期限切れ

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-04 |
| テスト名 | `test_verify_token_expired` |
| 前提条件 | `expires_in_hours = 0`（即期限切れ）で生成したトークン |
| 手順 | `verify_token(token, "secret")` を呼ぶ |
| 期待結果 | `Err(AppError::Unauthorized(_))` が返る |

#### TC-JWT-05: トークン検証 — 不正形式

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-05 |
| テスト名 | `test_verify_token_malformed` |
| 前提条件 | なし |
| 手順 | `verify_token("not.a.jwt", "secret")` を呼ぶ |
| 期待結果 | `Err(AppError::Unauthorized(_))` が返る |

#### TC-JWT-06: Claims のロールが保持される

| 項目 | 内容 |
|---|---|
| テストID | TC-JWT-06 |
| テスト名 | `test_token_preserves_admin_role` |
| 前提条件 | `UserRole::Admin` でトークン生成 |
| 手順 | 生成後に `verify_token` で復元する |
| 期待結果 | `claims.role == UserRole::Admin` |

---

### 1.2 エラー型（`src/errors.rs`）

#### TC-ERR-01: `diesel::result::Error::NotFound` → AppError::NotFound に変換

| 項目 | 内容 |
|---|---|
| テストID | TC-ERR-01 |
| テスト名 | `test_diesel_not_found_converts_to_app_error` |
| 手順 | `AppError::from(diesel::result::Error::NotFound)` を呼ぶ |
| 期待結果 | `AppError::NotFound(_)` |

#### TC-ERR-02: AppError の HTTP レスポンスマッピング

| AppError | 期待HTTPステータス |
|---|---|
| `Unauthorized` | 401 |
| `Forbidden` | 403 |
| `NotFound` | 404 |
| `BadRequest` | 400 |
| `Internal` | 500 |

テスト名: `test_error_response_status_codes`
手順: 各 `AppError` バリアントに対して `error_response()` を呼ぶ

#### TC-ERR-03: `Internal` エラーはクライアントに詳細を返さない

| 項目 | 内容 |
|---|---|
| テストID | TC-ERR-03 |
| テスト名 | `test_internal_error_hides_detail` |
| 手順 | `AppError::Internal("db password is secret".into()).error_response()` |
| 期待結果 | レスポンスボディに `"Internal server error"` と表示され `"db password"` 等の内部情報が含まれない |

---

## 2. 統合テスト

### 前提: テスト用アプリケーション構築

```rust
async fn build_test_app() -> impl Service<...> {
    // テスト用DB接続プール
    // マイグレーション実行
    // actix_web::test::init_service(App::new()...)
}
```

各テスト開始前にトランザクションを開始し、終了後にロールバック（データ汚染防止）。

---

### 2.1 認証 `POST /auth/login`

#### TC-AUTH-01: ログイン成功

| 項目 | 内容 |
|---|---|
| テストID | TC-AUTH-01 |
| テスト名 | `test_login_success` |
| 前提条件 | DBに `admin@example.com` / `changeme` のユーザーが存在 |
| リクエスト | `POST /auth/login` `{"email":"admin@example.com","password":"changeme"}` |
| 期待ステータス | 200 |
| 期待ボディ | `{ "token": "<string>", "user": { "email": "admin@example.com", "role": "Admin" } }` |
| 確認事項 | `password_hash` フィールドがレスポンスに含まれないこと |

#### TC-AUTH-02: メールアドレス不一致

| 項目 | 内容 |
|---|---|
| テストID | TC-AUTH-02 |
| テスト名 | `test_login_wrong_email` |
| リクエスト | `{"email":"notexist@example.com","password":"changeme"}` |
| 期待ステータス | 401 |
| 期待ボディ | `{"error": "Unauthorized: Invalid email or password"}` |

#### TC-AUTH-03: パスワード不一致

| 項目 | 内容 |
|---|---|
| テストID | TC-AUTH-03 |
| テスト名 | `test_login_wrong_password` |
| リクエスト | `{"email":"admin@example.com","password":"wrongpass"}` |
| 期待ステータス | 401 |
| 期待ボディ | `{"error": "Unauthorized: Invalid email or password"}` |

#### TC-AUTH-04: リクエストボディ不正（フィールド欠落）

| 項目 | 内容 |
|---|---|
| テストID | TC-AUTH-04 |
| テスト名 | `test_login_missing_fields` |
| リクエスト | `{"email":"admin@example.com"}` （passwordなし） |
| 期待ステータス | 400 |

#### TC-AUTH-05: Content-Type なし（JSON以外）

| 項目 | 内容 |
|---|---|
| テストID | TC-AUTH-05 |
| テスト名 | `test_login_wrong_content_type` |
| リクエスト | Content-Type: `text/plain`、ボディ: `email=admin&password=changeme` |
| 期待ステータス | 400 |

---

### 2.2 ユーザー管理

#### TC-USER-01: ユーザー登録 — 正常系（Admin）

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-01 |
| テスト名 | `test_create_user_as_admin` |
| 前提条件 | Adminトークンを取得済み |
| リクエスト | `POST /users` `{"name":"田中太郎","email":"tanaka@example.com","password":"pass1234","role":"User"}` |
| 期待ステータス | 201 |
| 期待ボディ | `{"id":...,"name":"田中太郎","email":"tanaka@example.com","role":"User","created_at":...}` |
| 確認事項 | `password_hash` がレスポンスに含まれないこと |

#### TC-USER-02: ユーザー登録 — ロールのデフォルト値

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-02 |
| テスト名 | `test_create_user_default_role` |
| リクエスト | `{"name":"花子","email":"hanako@example.com","password":"pass"}` （roleなし） |
| 期待ステータス | 201 |
| 期待ボディ | `"role": "User"` |

#### TC-USER-03: ユーザー登録 — Email重複

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-03 |
| テスト名 | `test_create_user_duplicate_email` |
| 前提条件 | `tanaka@example.com` が既に存在する |
| リクエスト | 同じメールアドレスで再度 `POST /users` |
| 期待ステータス | 400 |
| 期待ボディ | `{"error": "Bad request: Email already exists"}` |

#### TC-USER-04: ユーザー登録 — 認証なし

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-04 |
| テスト名 | `test_create_user_no_auth` |
| リクエスト | Authorizationヘッダーなしで `POST /users` |
| 期待ステータス | 401 |

#### TC-USER-05: ユーザー登録 — Userロールによるアクセス

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-05 |
| テスト名 | `test_create_user_as_non_admin` |
| 前提条件 | `UserRole::User` のトークンを使用 |
| 期待ステータス | 403 |

#### TC-USER-06: ユーザー一覧取得 — Admin

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-06 |
| テスト名 | `test_list_users_as_admin` |
| 前提条件 | DBに複数ユーザーが存在 |
| リクエスト | `GET /users` with Adminトークン |
| 期待ステータス | 200 |
| 期待ボディ | `[{...}, {...}]`（配列）、各要素に `password_hash` なし |

#### TC-USER-07: ユーザー一覧取得 — 非Admin

| 項目 | 内容 |
|---|---|
| テストID | TC-USER-07 |
| テスト名 | `test_list_users_as_non_admin` |
| 期待ステータス | 403 |

---

### 2.3 勘定項目管理

#### TC-CAT-01: 勘定項目作成 — 正常系

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-01 |
| テスト名 | `test_create_category_success` |
| リクエスト | `POST /categories` `{"name":"交通費"}` with Adminトークン |
| 期待ステータス | 201 |
| 期待ボディ | `{"id":...,"name":"交通費"}` |

#### TC-CAT-02: 勘定項目作成 — 空白名

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-02 |
| テスト名 | `test_create_category_empty_name` |
| リクエスト | `{"name":"   "}` （空白のみ） |
| 期待ステータス | 400 |
| 期待ボディ | `{"error": "Bad request: Category name cannot be empty"}` |

#### TC-CAT-03: 勘定項目作成 — 名前がトリムされる

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-03 |
| テスト名 | `test_create_category_trims_whitespace` |
| リクエスト | `{"name":"  交通費  "}` |
| 期待ステータス | 201 |
| 期待ボディ | `"name": "交通費"` （前後空白が除去されている） |

#### TC-CAT-04: 勘定項目作成 — 認証なし

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-04 |
| テスト名 | `test_create_category_no_auth` |
| 期待ステータス | 401 |

#### TC-CAT-05: 勘定項目一覧取得 — 名前昇順

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-05 |
| テスト名 | `test_list_categories_sorted_by_name` |
| 前提条件 | 「宿泊費」「交通費」「接待費」の順で登録 |
| 期待ボディ | 「交通費」「宿泊費」「接待費」の昇順で返る |

#### TC-CAT-06: 勘定項目一覧 — 非Admin

| 項目 | 内容 |
|---|---|
| テストID | TC-CAT-06 |
| テスト名 | `test_list_categories_as_non_admin` |
| 期待ステータス | 403 |

---

### 2.4 経費申請 CRUD

#### TC-EXP-01: 経費申請作成 — 正常系（ファイルなし）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-01 |
| テスト名 | `test_create_expense_success_without_file` |
| 前提条件 | UserトークンとカテゴリIDが存在 |
| リクエスト | `POST /expenses` multipart: `category_id`, `amount=5000`, `purpose=交通費`, `occurred_at=2026-01-15` |
| 期待ステータス | 201 |
| 期待ボディ | `{"status":"Pending","amount":5000,"purpose":"交通費","receipt_file":null,...}` |
| 確認事項 | `user_id` がトークンのユーザーIDと一致すること |

#### TC-EXP-02: 経費申請作成 — ファイルアップロード付き（JPEG）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-02 |
| テスト名 | `test_create_expense_with_jpeg` |
| リクエスト | multipartに `receipt` フィールドとして JPEG ファイルを添付 |
| 期待ステータス | 201 |
| 期待ボディ | `"receipt_file": "<uuid>.jpg"` |
| 確認事項 | `UPLOAD_DIR/<uuid>.jpg` が実際に作成されていること |

#### TC-EXP-03: 経費申請作成 — ファイルアップロード付き（PDF）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-03 |
| テスト名 | `test_create_expense_with_pdf` |
| 期待ボディ | `"receipt_file": "<uuid>.pdf"` |

#### TC-EXP-04: 経費申請作成 — 不正ファイル形式

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-04 |
| テスト名 | `test_create_expense_invalid_file_type` |
| リクエスト | `receipt` に `.txt` ファイルを添付（`text/plain`） |
| 期待ステータス | 400 |
| 期待ボディ | `"Unsupported file type"` を含むエラーメッセージ |

#### TC-EXP-05: 経費申請作成 — ファイルサイズ超過（10MB超）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-05 |
| テスト名 | `test_create_expense_file_too_large` |
| リクエスト | 10MB超のJPEGを添付 |
| 期待ステータス | 400 |
| 確認事項 | ファイルシステムに中途半端なファイルが残らないこと（クリーンアップ） |

#### TC-EXP-06: 経費申請作成 — 金額0以下

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-06 |
| テスト名 | `test_create_expense_zero_amount` |
| リクエスト | `amount=0` |
| 期待ステータス | 400 |
| 期待ボディ | `"amount must be a positive integer"` |

#### TC-EXP-07: 経費申請作成 — 必須フィールド欠落

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-07 |
| テスト名 | `test_create_expense_missing_required_fields` |
| サブケース | `category_id`なし / `purpose`なし / `occurred_at`なし の3パターン |
| 期待ステータス | 400（各ケース） |

#### TC-EXP-08: 経費申請作成 — 認証なし

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-08 |
| テスト名 | `test_create_expense_no_auth` |
| 期待ステータス | 401 |

#### TC-EXP-09: 経費申請一覧 — User は自分のものだけ取得

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-09 |
| テスト名 | `test_list_expenses_user_sees_own_only` |
| 前提条件 | UserA と UserB 両方の申請がDBに存在 |
| リクエスト | `GET /expenses` with UserAトークン |
| 期待ボディ | UserAの申請のみ返る（UserBのものは含まれない） |

#### TC-EXP-10: 経費申請一覧 — Admin は全員分取得

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-10 |
| テスト名 | `test_list_expenses_admin_sees_all` |
| 期待ボディ | 全ユーザーの申請が返る |

#### TC-EXP-11: 経費申請一覧 — 作成日時降順

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-11 |
| テスト名 | `test_list_expenses_ordered_by_created_at_desc` |
| 前提条件 | 複数の申請を時系列順に作成 |
| 期待ボディ | 最新の申請が先頭に来る |

#### TC-EXP-12: 経費申請更新 — 正常系（Pending）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-12 |
| テスト名 | `test_update_expense_success` |
| 前提条件 | Pending状態の自分の申請が存在 |
| リクエスト | `PATCH /expenses/{id}` `{"amount":8000,"purpose":"新幹線代"}` |
| 期待ステータス | 200 |
| 期待ボディ | `"amount":8000,"purpose":"新幹線代"` |

#### TC-EXP-13: 経費申請更新 — Approved 申請は編集不可

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-13 |
| テスト名 | `test_update_expense_approved_fails` |
| 前提条件 | Approved状態の申請が存在 |
| 期待ステータス | 400 |
| 期待ボディ | `"Only pending expenses can be edited"` |

#### TC-EXP-14: 経費申請更新 — 他人の申請は編集不可

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-14 |
| テスト名 | `test_update_expense_other_user_fails` |
| 前提条件 | UserBの申請に対してUserAのトークンでアクセス |
| 期待ステータス | 403 |
| 期待ボディ | `"Not your expense"` |

#### TC-EXP-15: 経費申請更新 — 存在しないID

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-15 |
| テスト名 | `test_update_expense_not_found` |
| リクエスト | ランダムUUIDで `PATCH /expenses/{random-uuid}` |
| 期待ステータス | 404 |

#### TC-EXP-16: 経費申請削除 — 正常系（Pending）

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-16 |
| テスト名 | `test_delete_expense_success` |
| 前提条件 | Pending状態の自分の申請 |
| リクエスト | `DELETE /expenses/{id}` |
| 期待ステータス | 204 |
| 確認事項 | 削除後に `GET /expenses` で該当IDが返らないこと |

#### TC-EXP-17: 経費申請削除 — Approved 申請は削除不可

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-17 |
| テスト名 | `test_delete_expense_approved_fails` |
| 期待ステータス | 400 |
| 期待ボディ | `"Only pending expenses can be deleted"` |

#### TC-EXP-18: 経費申請削除 — 他人の申請は削除不可

| 項目 | 内容 |
|---|---|
| テストID | TC-EXP-18 |
| テスト名 | `test_delete_expense_other_user_fails` |
| 期待ステータス | 403 |

---

### 2.5 承認フロー

#### TC-APR-01: 個別承認 — Pending → Approved

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-01 |
| テスト名 | `test_update_status_pending_to_approved` |
| 前提条件 | Pending状態の申請、Adminトークン |
| リクエスト | `PATCH /expenses/{id}/status` `{"status":"Approved"}` |
| 期待ステータス | 200 |
| 期待ボディ | `"status":"Approved"` |

#### TC-APR-02: 個別差し戻し — Pending → Rejected

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-02 |
| テスト名 | `test_update_status_pending_to_rejected` |
| リクエスト | `{"status":"Rejected"}` |
| 期待ステータス | 200 |
| 期待ボディ | `"status":"Rejected"` |

#### TC-APR-03: ステータス巻き戻し — Approved → Pending

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-03 |
| テスト名 | `test_update_status_revert_to_pending` |
| リクエスト | Approved状態の申請に `{"status":"Pending"}` |
| 期待ステータス | 200 |
| 期待ボディ | `"status":"Pending"` |

#### TC-APR-04: ステータス更新 — 非Adminは操作不可

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-04 |
| テスト名 | `test_update_status_as_non_admin` |
| 期待ステータス | 403 |

#### TC-APR-05: ステータス更新 — 不正なステータス値

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-05 |
| テスト名 | `test_update_status_invalid_value` |
| リクエスト | `{"status":"Unknown"}` |
| 期待ステータス | 400 |

#### TC-APR-06: ステータス更新 — 存在しないID

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-06 |
| テスト名 | `test_update_status_not_found` |
| 期待ステータス | 404 |

#### TC-APR-07: 一括承認 — 正常系（Pending複数件）

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-07 |
| テスト名 | `test_bulk_approve_success` |
| 前提条件 | Pending状態の申請3件 |
| リクエスト | `POST /expenses/bulk-approve` `{"ids":["<id1>","<id2>","<id3>"]}` |
| 期待ステータス | 200 |
| 期待ボディ | `{"approved":3}` |
| 確認事項 | 3件すべてが `Approved` になっていること |

#### TC-APR-08: 一括承認 — Pending以外は対象外

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-08 |
| テスト名 | `test_bulk_approve_skips_non_pending` |
| 前提条件 | Pending1件 + Approved1件 + Rejected1件の計3件のIDを送信 |
| 期待ボディ | `{"approved":1}` （Pendingの1件のみカウント） |

#### TC-APR-09: 一括承認 — IDリスト空

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-09 |
| テスト名 | `test_bulk_approve_empty_ids` |
| リクエスト | `{"ids":[]}` |
| 期待ステータス | 400 |
| 期待ボディ | `"ids must not be empty"` |

#### TC-APR-10: 一括承認 — 非Admin

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-10 |
| テスト名 | `test_bulk_approve_as_non_admin` |
| 期待ステータス | 403 |

#### TC-APR-11: 一括承認 — 存在しないIDは無視される

| 項目 | 内容 |
|---|---|
| テストID | TC-APR-11 |
| テスト名 | `test_bulk_approve_ignores_nonexistent_ids` |
| リクエスト | 存在しないUUIDを含む `ids` リスト |
| 期待ステータス | 200 |
| 期待ボディ | `{"approved":0}` |

---

### 2.6 認証ミドルウェア

#### TC-MID-01: Authorizationヘッダーなし

| 項目 | 内容 |
|---|---|
| テストID | TC-MID-01 |
| テスト名 | `test_middleware_no_auth_header` |
| 手順 | ヘッダーなしで認証必須エンドポイントにアクセス |
| 期待ステータス | 401 |
| 期待ボディ | `"Authorization header missing"` |

#### TC-MID-02: `Bearer ` プレフィックスなし

| 項目 | 内容 |
|---|---|
| テストID | TC-MID-02 |
| テスト名 | `test_middleware_no_bearer_prefix` |
| リクエストヘッダー | `Authorization: <token>` （Bearerなし） |
| 期待ステータス | 401 |
| 期待ボディ | `"Bearer token required"` |

#### TC-MID-03: 期限切れトークン

| 項目 | 内容 |
|---|---|
| テストID | TC-MID-03 |
| テスト名 | `test_middleware_expired_token` |
| リクエストヘッダー | `Authorization: Bearer <expired-token>` |
| 期待ステータス | 401 |

---

## 3. テストカバレッジ目標

| モジュール | 目標カバレッジ |
|---|---|
| `auth/jwt.rs` | 100% |
| `auth/middleware.rs` | 90%以上 |
| `handlers/auth.rs` | 90%以上 |
| `handlers/users.rs` | 90%以上 |
| `handlers/categories.rs` | 90%以上 |
| `handlers/expenses.rs` | 85%以上 |
| `errors.rs` | 100% |

---

## 4. テスト実行方法

```bash
# 単体テストのみ
cargo test --lib

# 統合テストのみ（テスト用DB要）
TEST_DATABASE_URL=postgres://... cargo test --test integration

# 全テスト
cargo test

# カバレッジ（cargo-llvm-cov）
cargo llvm-cov --html
```

---

## 5. テストデータ定義

```rust
// テスト用フィクスチャ
pub struct TestFixtures {
    pub admin_token: String,       // AdminロールのJWT
    pub user_token: String,        // UserロールのJWT
    pub user_b_token: String,      // 別ユーザーのJWT（権限テスト用）
    pub category_id: Uuid,         // 作成済みカテゴリID
    pub pending_expense_id: Uuid,  // Pending状態の経費申請ID
    pub approved_expense_id: Uuid, // Approved状態の経費申請ID
}
```
