# Backend (Actix-web / Diesel) 開発ルール＆学び

---

## Diesel

### JOIN でのレコード取得

```rust
let rows: Vec<(Expense, User)> = expenses::table
    .inner_join(users::table)
    .select((Expense::as_select(), User::as_select()))
    .order(expenses::created_at.desc())
    .load(&mut conn)
    .map_err(AppError::from)?;
```

### returning / as_returning

INSERT/UPDATE の結果を取得するには `.returning(T::as_returning()).get_result()` を使う。

```rust
let expense: Expense = diesel::insert_into(expenses::table)
    .values(&new_expense)
    .returning(Expense::as_returning())
    .get_result(&mut conn)
    .map_err(AppError::from)?;
```

---

## Actix-web

### multipart ファイルアップロード

ファイルを含むエンドポイントは JSON ではなく `multipart/form-data` で受け取る。
E2E テストの API ヘルパーも `data:` ではなく `multipart:` を指定すること。

```rust
#[post("/expenses")]
pub async fn create_expense(
    multipart: Multipart,
    ...
) -> Result<HttpResponse, AppError> { ... }
```

### Content-Disposition とクロスオリジンダウンロード

HTML の `download` 属性はクロスオリジン URL では**ブラウザに無視される**。
強制ダウンロードさせるにはサーバー側で `attachment` を返す必要がある。

```rust
// inline  → ブラウザ内で表示
// attachment → クロスオリジンでも強制ダウンロード
.append_header((
    "Content-Disposition",
    format!("attachment; filename=\"{}\"", filename),
))
```

### パストラバーサル防御

ファイル名に `..` `/` `\` が含まれていたら即 BadRequest を返す。

```rust
if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
    return Err(AppError::BadRequest("Invalid filename".into()));
}
```

### CORS 設定

フロントエンドが別オリジン（例: localhost:3000）の場合、
`actix-cors` で明示的に許可しないと全リクエストがブロックされる。

```rust
use actix_cors::Cors;
let cors = Cors::default()
    .allowed_origin("http://localhost:3000")
    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
    .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE]);
```
