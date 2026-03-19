# commit & push

未コミットの変更を論理的なグループに分割し、適切なコミットメッセージを付けてコミット＆プッシュする。

## 実行手順

以下のステップを順番に実行してください。

---

### Step 1: 変更内容の把握

1. `git status` で未コミットファイルを確認する
2. `git diff HEAD` で変更内容を確認する
3. `git log --oneline -10` で既存のコミットメッセージスタイルを確認する

---

### Step 2: 変更をグループに分類する

以下の観点でファイルをグループ化する:

| グループ | 対象パス | prefix の例 |
|---------|---------|------------|
| API ハンドラ | `src/handlers/` | `feat(api):` / `fix(api):` |
| 認証 | `src/auth/` | `feat(auth):` / `fix(auth):` |
| ドメインモデル / スキーマ | `src/models/`, `src/schema.rs` | `feat(domain):` / `fix(domain):` |
| コア（エラー / DB / 設定） | `src/errors.rs`, `src/db.rs`, `src/config.rs`, `src/upload.rs`, `src/main.rs` | `feat(core):` / `fix(core):` |
| マイグレーション | `migrations/` | `feat(db):` / `fix(db):` |
| Docker / インフラ | `Dockerfile`, `docker-compose.yml`, `.dockerignore` | `chore(docker):` |
| 依存クレート | `Cargo.toml`, `Cargo.lock` | `chore(deps):` |
| CI | `.github/workflows/` | `ci:` |
| 設定・環境 | `diesel.toml`, `.env.example`, `.gitignore` | `chore:` |
| ドキュメント | `README.md`, `docs/`, `LICENSE` | `docs:` |

- 同一機能に関連する複数層の変更は 1 コミットにまとめてよい
- 無関係な変更は必ず別コミットにする

---

### Step 3: コミットメッセージを決定する

各グループのコミットメッセージは以下のルールに従う:

- **Conventional Commits** 形式: `type(scope): description`
- 説明は「何をしたか」ではなく「なぜ・何のためか」を書く
- `git log` の言語（英語）に合わせて **英語** で記述する
- 必ず末尾に付与: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
- HEREDOC 形式でコミットメッセージを渡す

---

### Step 4: グループ順にステージ → コミットを実行する

依存関係のある順（Domain → Core → Auth → API → DB → Infra → CI → Docs）でコミットする。
各コミット後に成功を確認してから次へ進む。

---

### Step 5: プッシュする

```bash
git push origin main
```

---

### Step 6: 完了報告

- 作成したコミット一覧（hash + メッセージ）をまとめて報告する
- 除外したファイルがある場合はその理由も報告する

---

## 安全ルール

- `.env` / 秘密情報を含むファイルは絶対にコミットしない（`.gitignore` で除外済みだが目視確認すること）
- `--force` push は絶対に使わない
- 既存コミットを `--amend` しない
- `uploads/` 配下の実ファイルはコミットしない（`.gitkeep` のみ許可）
- 不明なファイルはスキップしてユーザーに報告する
