# rust-expense-api

![CI](https://github.com/Hiroshi-Kazui/rust-expense-api/actions/workflows/ci.yml/badge.svg)

中小企業向け **経費申請・承認管理システム**。

PHPエンジニアとして20年超の実務経験を持つ筆者が、Rustの学習目的で実装したプロジェクトです。
バックエンド（actix-web）からフロントエンド（Leptos/WASM）まで、**フルRustスタック**で構成しています。

---

## 技術スタック

### バックエンド

| 技術 | 選定理由 |
|---|---|
| **Rust** | 型安全性・パフォーマンス・メモリ安全性の学習 |
| **actix-web** | Rustで最も実績のある非同期WebフレームワークかつTechEmpowerベンチマーク上位 |
| **Diesel** | コンパイル時SQLチェックが可能な型安全ORMで、PHPのEloquent的な安心感 |
| **PostgreSQL** | 本番実績豊富なRDBMS。ENUM型・UUID対応 |
| **JWT** | ステートレス認証で水平スケールに対応 |

### フロントエンド

| 技術 | 選定理由 |
|---|---|
| **Leptos** | RustでWebUIが書ける宣言的リアクティブフレームワーク |
| **WebAssembly (WASM)** | RustコードをブラウザでそのままCSRとして実行 |
| **Tailwind CSS** | ユーティリティファーストでデザインを素早く構築 |
| **trunk** | WASMアプリのビルド・バンドルツール |

### インフラ

| 技術 | 選定理由 |
|---|---|
| **Docker Compose** | `docker compose up` 一発でフロント・バック・DBが全起動 |
| **nginx** | フロントエンド静的ファイルの配信・SPAルーティング対応 |

---

## 起動手順

### 前提条件

- Docker / Docker Compose が使えること（ローカルにRust・Node.jsは不要）

### 1. リポジトリをクローン

```bash
git clone https://github.com/Hiroshi-Kazui/rust-expense-api.git
cd rust-expense-api
```

### 2. 起動

```bash
docker compose up --build
```

初回ビルドは10〜20分かかります（WASM + バックエンドのRustビルドを含むため）。
2回目以降はDockerレイヤーキャッシュで高速化されます。

起動後、以下が自動で行われます：

1. PostgreSQL が起動・ヘルスチェック通過
2. Diesel マイグレーション実行（テーブル作成）
3. 初期 admin ユーザーと勘定項目を自動シード（初回のみ）

### 3. アクセス

| サービス | URL |
|---|---|
| **フロントエンド（UI）** | http://localhost:3000 |
| **バックエンド（API）** | http://localhost:8080 |

ログイン情報（初期値）：

| 項目 | 値 |
|---|---|
| メールアドレス | `admin@example.com` |
| パスワード | `changeme` |

### 4. フロントエンドのローカル開発（trunk）

Dockerを使わずにフロントエンドのみ開発する場合：

```bash
# 前提: Rust + wasm32ターゲット + trunk + Node.js が必要
rustup target add wasm32-unknown-unknown
cargo install trunk

cd frontend
npm install          # Tailwind CSS のインストール
trunk serve          # http://localhost:8888 でホットリロード起動
```

バックエンドは別途 `docker compose up db app` で起動しておく必要があります。

### 5. API 動作確認

```bash
curl http://localhost:8080/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"changeme"}'
```

### 環境変数（オプション）

`docker-compose.yml` の `environment` セクションで上書き可能：

| 変数 | デフォルト | 説明 |
|---|---|---|
| `ADMIN_EMAIL` | `admin@example.com` | 初期adminのメールアドレス |
| `ADMIN_PASSWORD` | `changeme` | 初期adminのパスワード |
| `JWT_SECRET` | （設定必須） | JWTの署名キー |
| `JWT_EXPIRES_IN_HOURS` | `24` | トークン有効期限（時間） |
| `UPLOAD_DIR` | `/app/uploads` | ファイル保存先 |

---

## API エンドポイント使用例

### 認証

#### POST /auth/login

```bash
curl -s -X POST http://localhost:8080/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"changeme"}' | jq .
```

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Administrator",
    "email": "admin@example.com",
    "role": "Admin",
    "created_at": "2026-01-01T00:00:00"
  }
}
```

以降のリクエストでは取得したトークンを使用します：

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

---

### メンバー管理（admin のみ）

#### POST /users — メンバー登録

```bash
curl -s -X POST http://localhost:8080/users \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "田中 太郎",
    "email": "tanaka@example.com",
    "password": "secure-password",
    "role": "User"
  }' | jq .
```

#### GET /users — メンバー一覧

```bash
curl -s http://localhost:8080/users \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

### 勘定項目（admin のみ）

#### POST /categories — 勘定項目作成

```bash
curl -s -X POST http://localhost:8080/categories \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name": "交通費"}' | jq .
```

#### GET /categories — 勘定項目一覧

```bash
curl -s http://localhost:8080/categories \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

### 経費申請（user）

#### POST /expenses — 申請作成（レシートなし）

```bash
curl -s -X POST http://localhost:8080/expenses \
  -H "Authorization: Bearer $USER_TOKEN" \
  -F "category_id=<カテゴリUUID>" \
  -F "amount=3500" \
  -F "purpose=東京-大阪 新幹線代" \
  -F "occurred_at=2026-01-15"
```

#### POST /expenses — 申請作成（レシート画像付き）

```bash
curl -s -X POST http://localhost:8080/expenses \
  -H "Authorization: Bearer $USER_TOKEN" \
  -F "category_id=<カテゴリUUID>" \
  -F "amount=15000" \
  -F "purpose=クライアント接待" \
  -F "occurred_at=2026-01-20" \
  -F "note=4名参加" \
  -F "receipt=@/path/to/receipt.jpg"
```

#### GET /expenses — 自分の申請一覧

```bash
curl -s http://localhost:8080/expenses \
  -H "Authorization: Bearer $USER_TOKEN" | jq .
```

#### PATCH /expenses/{id} — 申請編集（pending のみ）

```bash
curl -s -X PATCH http://localhost:8080/expenses/<経費UUID> \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"amount": 4000, "purpose": "東京-大阪 新幹線代（変更）"}' | jq .
```

#### DELETE /expenses/{id} — 申請削除（pending のみ）

```bash
curl -s -X DELETE http://localhost:8080/expenses/<経費UUID> \
  -H "Authorization: Bearer $USER_TOKEN"
```

---

### 承認フロー（admin のみ）

#### GET /expenses — 全申請一覧

```bash
curl -s http://localhost:8080/expenses \
  -H "Authorization: Bearer $TOKEN" | jq .
```

#### PATCH /expenses/{id}/status — 個別承認・差し戻し

```bash
# 承認
curl -s -X PATCH http://localhost:8080/expenses/<経費UUID>/status \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"status": "Approved"}' | jq .

# 差し戻し
curl -s -X PATCH http://localhost:8080/expenses/<経費UUID>/status \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"status": "Rejected"}' | jq .
```

#### POST /expenses/bulk-approve — 一括承認

```bash
curl -s -X POST http://localhost:8080/expenses/bulk-approve \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "ids": [
      "<経費UUID-1>",
      "<経費UUID-2>",
      "<経費UUID-3>"
    ]
  }' | jq .
```

```json
{ "approved": 3 }
```

---

## データモデル

```
users
  id (UUID PK), name, email, password_hash, role (admin|user), created_at

categories
  id (UUID PK), name

expenses
  id (UUID PK), user_id (FK), category_id (FK),
  amount, purpose, occurred_at, note, receipt_file,
  status (pending|approved|rejected), created_at
```

---

## ファイルアップロード仕様

- 対応形式：JPEG・PNG・PDF
- 最大サイズ：10 MB
- 保存先：コンテナ内 `/app/uploads/`（Dockerボリュームで永続化）
- ファイル名：UUIDで自動リネーム（パストラバーサル対策）
- アクセス：`GET /uploads/<ファイル名>` で直接取得可能

---

## フロントエンド画面一覧

### 画面構成

| パス | 画面名 | 権限 |
|---|---|---|
| `/login` | ログイン | 全員（未認証） |
| `/expenses` | 経費申請一覧 | 全ユーザー |
| `/expenses/new` | 経費申請 新規作成 | 全ユーザー |
| `/expenses/:id/edit` | 経費申請 編集 | 本人のみ（Pending のみ編集可） |
| `/admin/expenses` | 全申請一覧（管理者） | Admin のみ |
| `/admin/users` | ユーザー管理 | Admin のみ |
| `/admin/categories` | 勘定項目管理 | Admin のみ |

### 各画面の機能

#### ログイン（`/login`）

- メールアドレス・パスワードによる認証
- 認証成功後 `/expenses` へリダイレクト
- 未入力・認証失敗時のエラーメッセージ表示
- JWT トークンを `localStorage` に保存

#### 経費申請一覧（`/expenses`）

- 自分が作成した申請をテーブル表示（目的・金額・発生日・ステータス）
- ステータスバッジ表示（申請中 / 承認済 / 差し戻し）
- Pending 申請のみ「編集」リンクを表示
- 「新規申請」ボタンで作成画面へ遷移

#### 経費申請 新規作成（`/expenses/new`）

- 勘定項目（セレクト）・金額・目的・発生日・備考を入力
- レシート・領収書をファイル添付（JPEG / PNG / PDF、最大 10 MB）
- 必須項目未入力・金額不正時のバリデーションエラー表示
- 申請成功後 `/expenses` へリダイレクト

#### 経費申請 編集（`/expenses/:id/edit`）

- 既存データをフォームに初期表示
- 勘定項目・金額・目的・発生日・備考を変更して更新
- 削除ボタンは **2回クリック確認方式**（1回目で「本当に削除」に変わる）
- Pending 以外の申請は編集・削除不可

#### 全申請一覧 — 管理者（`/admin/expenses`）

- 全ユーザーの申請を一覧表示
- Pending 申請に「承認」「差し戻し」ボタンを表示
- 承認済・差し戻し済の申請は「戻す」ボタンで Pending に戻す
- Pending 申請にチェックボックスを表示し、複数選択して**一括承認**が可能
- ステータス変更はページリロードなしに即時反映

#### ユーザー管理（`/admin/users`）

- 登録済みユーザー一覧をテーブル表示
- 名前・メールアドレス・パスワード・ロール（User / Admin）を入力してユーザー登録

#### 勘定項目管理（`/admin/categories`）

- 勘定項目の一覧表示
- 名前を入力して勘定項目を追加

### 共通レイアウト

- 左サイドバーにナビゲーションリンクとログインユーザー名を表示
- Admin ユーザーのみ管理メニュー（全申請一覧・ユーザー・勘定項目）を表示
- 未認証状態で保護ページにアクセスすると `/login` へリダイレクト
- Admin 以外が管理ページにアクセスすると `/expenses` へリダイレクト

---

## プロジェクト構成

```
rust-expense-api/
├── Cargo.toml
├── Dockerfile              # マルチステージビルド（依存レイヤーキャッシュ）
├── .dockerignore
├── docker-compose.yml
├── migrations/             # Diesel マイグレーション
├── src/
│   ├── main.rs
│   ├── config.rs           # 環境変数管理
│   ├── db.rs               # r2d2 接続プール
│   ├── errors.rs           # 統一エラー型
│   ├── schema.rs           # Diesel スキーマ定義
│   ├── upload.rs           # ファイルアップロード処理
│   ├── seeder.rs           # 起動時の初期データ投入
│   ├── auth/
│   │   ├── jwt.rs          # JWT生成・検証
│   │   └── middleware.rs   # actix-web 認証ガード
│   ├── models/             # DB モデル・リクエスト型
│   └── handlers/           # ルートハンドラ
├── frontend/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs         # WASMエントリポイント
│   │   ├── router.rs       # SPA ルーティング・認証ガード
│   │   ├── api.rs          # バックエンドAPIクライアント
│   │   ├── store.rs        # 認証ストア（localStorage連携）
│   │   ├── types.rs        # 共有型定義
│   │   ├── error.rs        # フロントエンドエラー型
│   │   ├── components/
│   │   │   ├── layout.rs   # サイドバー付きレイアウト
│   │   │   └── toast.rs    # トースト通知
│   │   └── pages/
│   │       ├── login.rs
│   │       ├── users.rs
│   │       ├── categories.rs
│   │       ├── admin_expenses.rs
│   │       └── expenses/
│   │           ├── list.rs
│   │           ├── create.rs
│   │           └── edit.rs
│   ├── Dockerfile
│   └── nginx.conf          # SPA ルーティング対応
├── e2e/                    # Playwright E2E テスト
│   ├── playwright.config.ts
│   └── tests/
└── .github/workflows/ci.yml
```

---

## 実装者について

PHPエンジニアとして20年超、Laravel・Symfony等を用いたWebシステム開発に従事。
本プロジェクトはRustの学習目的で実装しました。

PHPとの比較で感じたRustの特徴：

- **所有権・借用** — 実行時エラーの多くをコンパイル時に検出できる
- **型システム** — `Option<T>` / `Result<T, E>` による明示的なエラーハンドリング（nullなし）
- **Diesel** — SQLをコンパイル時に型チェックする設計はEloquentにはない安心感
- **非同期** — `async/await` の考え方はPHPのFiberに近いが、より型安全
