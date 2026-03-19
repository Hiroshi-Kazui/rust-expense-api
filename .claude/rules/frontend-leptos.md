# Frontend (Leptos / WASM) 開発ルール＆学び

---

## Leptos 固有のクセ

### 1. `For` コンポーネントのキーは変化しうる状態を含める

`key` が同じだと Leptos は行を再利用してしまい、状態変化が画面に反映されない。

```rust
// NG: id だけでは status 変更後も行が再レンダリングされない
key=|e| e.id.clone()

// OK: 変化しうるフィールドを複合キーに含める
key=|e| format!("{}-{:?}", e.id, e.status)
```

### 2. move クロージャへの二重 move

同じ変数を複数の `move ||` クロージャに渡すとコンパイルエラーになる。
クロージャの数だけ事前に `.clone()` しておく。

```rust
let f_str = value.to_string();
let f_str2 = f_str.clone();

view! {
    <button
        class=move || if x == f_str { "active" } else { "inactive" }
        on:click=move |_| signal.set(f_str2.clone())
    >
```

### 3. 条件分岐で異なる View 型を返す場合は `.into_any()`

Leptos の型システムでは、分岐ごとに View 型が異なるとコンパイルエラーになる。
`.into_any()` で型を消去する。

```rust
{if is_pending {
    view! { <button>"承認"</button> }.into_any()
} else {
    view! { <span></span> }.into_any()
}}
```

### 4. `gloo_storage` と `leptos::prelude::*` のメソッド競合

`gloo_storage::Storage::get` と `leptos::prelude::Get::get` が衝突してコンパイルエラーになる場合がある。
明示的にインポートして解消する。

```rust
use leptos::prelude::Get;
```

### 5. クライアントサイドフィルタリング

`For` の `each` クロージャ内でフィルタすることで、
バックエンドの変更なしにリアクティブなフィルタリングが実現できる。

```rust
let status_filter = RwSignal::new("all".to_string());

<For
    each=move || {
        let f = status_filter.get();
        expenses.get().into_iter()
            .filter(|e| f == "all" || format!("{:?}", e.status) == f)
            .collect::<Vec<_>>()
    }
    key=|e| format!("{}-{:?}", e.id, e.status)
    ...
/>
```

### 6. 非同期処理は `spawn_local`

`RwSignal` は `Send` でないためスレッドを跨げない。
非同期処理は必ず `wasm_bindgen_futures::spawn_local` を使う。

```rust
wasm_bindgen_futures::spawn_local(async move {
    match api::get::<T>("/endpoint").await {
        Ok(data) => signal.set(data),
        Err(e)   => error.set(Some(e.to_string())),
    }
});
```

### 7. HTML の `min` 属性とカスタムエラーメッセージの競合

`input[min="1"]` などのネイティブバリデーション属性があると、
ブラウザのネイティブバリデーションが先に発火し、
Leptos 側のエラーメッセージが表示されない。
バリデーションをコード側で完全に制御したい場合は `min` 属性を外す。

---

## wasm-bindgen-test

フロントエンド単体テストは `wasm32-unknown-unknown` でコンパイルされ、
ヘッドレスブラウザで実行される。

```rust
use wasm_bindgen_test::wasm_bindgen_test;
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_something() { ... }
```

実行コマンド:
```bash
wasm-pack test --headless --chrome
```

---

## E2E (Playwright) 注意点

- `locator('..')` で親 `<tr>` を取得できる。`locator('../..')` は `<tbody>` まで上がりすぎる。
- テストデータは `Date.now()` で一意な文字列を生成し、並列実行時のデータ衝突を防ぐ。
- 複数要素にマッチする locator は strict mode エラーになる。`.first()` かより具体的なセレクタを使う。
- Firefox は別途 `npx playwright install firefox` が必要。Chromium のみで運用する構成でもよい。
