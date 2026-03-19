use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::Serialize;
use crate::api;
use crate::components::layout::Layout;
use crate::types::{Category, Expense};

#[derive(Serialize)]
struct UpdateExpenseRequest {
    category_id: Option<String>,
    amount: Option<i32>,
    purpose: Option<String>,
    occurred_at: Option<String>,
    note: Option<String>,
}

#[component]
pub fn ExpenseEditPage() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let id = move || params.get().get("id").unwrap_or_default();

    let expense = RwSignal::new(Option::<Expense>::None);
    let categories = RwSignal::new(Vec::<Category>::new());
    let category_id = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let purpose = RwSignal::new(String::new());
    let occurred_at = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let receipt_file = RwSignal::new(Option::<String>::None);
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);
    let delete_confirm = RwSignal::new(false);

    // Fetch expense then categories sequentially
    let expense_id = id();
    wasm_bindgen_futures::spawn_local(async move {
        match api::get::<Expense>(&format!("/expenses/{}", expense_id)).await {
            Ok(e) => {
                category_id.set(e.category_id.clone());
                amount.set(e.amount.to_string());
                purpose.set(e.purpose.clone());
                occurred_at.set(e.occurred_at.get(..10).unwrap_or("").to_string());
                note.set(e.note.clone().unwrap_or_default());
                receipt_file.set(e.receipt_file.clone());
                expense.set(Some(e));
            }
            Err(e) => error_msg.set(Some(e.to_string())),
        }
        match api::get::<Vec<Category>>("/categories").await {
            Ok(data) => categories.set(data),
            Err(_) => {}
        }
    });

    let navigate_submit = navigate.clone();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let amt: Option<i32> = amount.get().parse().ok().filter(|&v| v > 0);
        let purp = purpose.get().trim().to_string();
        let date = occurred_at.get();
        let n = note.get().trim().to_string();
        loading.set(true);
        error_msg.set(None);
        let eid = id();
        let nav = navigate_submit.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateExpenseRequest {
                category_id: Some(category_id.get()),
                amount: amt,
                purpose: if purp.is_empty() { None } else { Some(purp) },
                occurred_at: if date.is_empty() { None } else { Some(date) },
                note: if n.is_empty() { None } else { Some(n) },
            };
            match api::patch::<_, Expense>(&format!("/expenses/{}", eid), &req).await {
                Ok(_) => nav("/expenses", Default::default()),
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    let on_delete = move |_| {
        if !delete_confirm.get() {
            delete_confirm.set(true);
            return;
        }
        loading.set(true);
        let eid = id();
        let nav = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_req(&format!("/expenses/{}", eid)).await {
                Ok(_) => nav("/expenses", Default::default()),
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    let on_delete_receipt = move |_| {
        loading.set(true);
        let eid = id();
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_req(&format!("/expenses/{}/receipt", eid)).await {
                Ok(_) => {
                    receipt_file.set(None);
                    loading.set(false);
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <Layout>
            <div class="max-w-lg">
                <h2 class="text-base font-bold text-gray-900 mb-4">"経費申請 編集"</h2>
                {move || error_msg.get().map(|e| view! { <p class="text-sm text-red-500 mb-3">{e}</p> })}
                {move || {
                    let on_submit = on_submit.clone();
                    let on_delete = on_delete.clone();
                    expense.get().map(|_| view! {
                    <div class="card">
                        <form on:submit=on_submit class="space-y-4">
                            <div>
                                <label class="label">"勘定項目"</label>
                                <select class="input-field"
                                    on:change=move |ev| category_id.set(event_target_value(&ev))>
                                    <For
                                        each=move || categories.get()
                                        key=|c| c.id.clone()
                                        children=move |cat| {
                                            let id = cat.id.clone();
                                            let selected = category_id.get() == id;
                                            view! {
                                                <option value={id} selected={selected}>{cat.name.clone()}</option>
                                            }
                                        }
                                    />
                                </select>
                            </div>
                            <div>
                                <label class="label">"金額（円）"</label>
                                <input type="number" class="input-field"
                                    on:input=move |ev| amount.set(event_target_value(&ev))
                                    prop:value=amount />
                            </div>
                            <div>
                                <label class="label">"目的・用途"</label>
                                <input type="text" class="input-field"
                                    on:input=move |ev| purpose.set(event_target_value(&ev))
                                    prop:value=purpose />
                            </div>
                            <div>
                                <label class="label">"経費発生日"</label>
                                <input type="date" class="input-field"
                                    on:input=move |ev| occurred_at.set(event_target_value(&ev))
                                    prop:value=occurred_at />
                            </div>
                            <div>
                                <label class="label">"備考・注記"</label>
                                <textarea class="input-field h-20 resize-none"
                                    on:input=move |ev| note.set(event_target_value(&ev))
                                    prop:value=note />
                            </div>
                            {move || receipt_file.get().map(|f| {
                                let view_url = crate::api::upload_url(&f);
                                let dl_url = crate::api::upload_url(&f);
                                let is_pending = expense.get()
                                    .map(|e| matches!(e.status, crate::types::ExpenseStatus::Pending))
                                    .unwrap_or(false);
                                view! {
                                    <div class="border border-gray-200 rounded p-3 bg-gray-50">
                                        <p class="text-xs text-gray-500 mb-2">"添付ファイル"</p>
                                        <div class="flex items-center gap-3">
                                            <span class="text-sm text-gray-700 truncate max-w-xs">{f.clone()}</span>
                                            <a href={view_url} target="_blank"
                                                class="text-xs text-blue-500 hover:text-blue-700 shrink-0">"表示"</a>
                                            <a href={dl_url} download={f}
                                                class="text-xs text-green-600 hover:text-green-800 shrink-0">"ダウンロード"</a>
                                            {if is_pending {
                                                view! {
                                                    <button type="button"
                                                        class="text-xs text-red-500 hover:text-red-700 shrink-0"
                                                        disabled=loading
                                                        on:click=on_delete_receipt>
                                                        "添付削除"
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            })}
                            {move || error_msg.get().map(|e| view! { <p class="text-xs text-red-500">{e}</p> })}
                            <div class="flex gap-2 justify-between pt-2">
                                <div class="flex gap-2">
                                    <button type="submit" class="btn-primary" disabled=loading>
                                        {move || if loading.get() { "更新中..." } else { "更新する" }}
                                    </button>
                                    <a href="/expenses" class="btn-secondary">"キャンセル"</a>
                                </div>
                                <button
                                    type="button"
                                    class="btn-danger"
                                    disabled=loading
                                    on:click=on_delete
                                >
                                    {move || if delete_confirm.get() { "本当に削除" } else { "削除" }}
                                </button>
                            </div>
                        </form>
                    </div>
                })}}
            </div>
        </Layout>
    }
}
