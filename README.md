# rust-expense-api

![CI](https://github.com/Hiroshi-Kazui/rust-expense-api/actions/workflows/ci.yml/badge.svg)

中小企業向け **経費申請・承認管理 REST API**。

PHPエンジニアとして20年超の実務経験を持つ筆者が、Rustの学習目的で実装したプロジェクトです。
actix-web を中心に、型システムと所有権モデルを活かした実務的なAPIサーバー構成を目指しました。

---

## 技術スタック

| 技術 | 選定理由 |
|---|---|
| **Rust** | 型安全性・パフォーマンス・メモリ安全性の学習 |
| **actix-web** | Rustで最も実績のある非同期WebフレームワークかつTechEmpowerベンチマーク上位 |
| **Diesel** | コンパイル時SQLチェックが可能な型安全ORMで、PHPのEloquent的な安心感 |
| **PostgreSQL** | 本番実績豊富なRDBMS。ENUM型・UUID対応 |
| **JWT** | ステートレス認証で水平スケールに対応 |
| **Docker Compose** | ローカル環境を問わず `docker compose up` 一発で動作 |

---

## 起動手順

### 前提条件

- Docker / Docker Compose が使えること（ローカルにRustは不要）

### 1. リポジトリをクローン

```bash
git clone https://github.com/Hiroshi-Kazui/rust-expense-api.git
cd rust-expense-api
```

### 2. 起動

```bash
docker compose up --build
```

初回ビルドは数分かかります（空の `main.rs` で依存クレートを先にビルドしてキャッシュするため、2回目以降は高速）。

起動後、以下が自動で行われます：

1. PostgreSQL が起動・ヘルスチェック通過
2. Diesel マイグレーション実行（テーブル作成）
3. 初期 admin ユーザーを自動シード（初回のみ）

### 3. 動作確認

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
│   ├── auth/
│   │   ├── jwt.rs          # JWT生成・検証
│   │   └── middleware.rs   # actix-web 認証ガード
│   ├── models/             # DB モデル・リクエスト型
│   └── handlers/           # ルートハンドラ
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
