use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;
use crate::api;
use crate::components::layout::Layout;
use crate::types::{Category, Expense};

#[component]
pub fn ExpenseCreatePage() -> impl IntoView {
    let navigate = use_navigate();
    let categories = RwSignal::new(Vec::<Category>::new());
    let category_id = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let purpose = RwSignal::new(String::new());
    let occurred_at = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    // Fetch categories
    wasm_bindgen_futures::spawn_local(async move {
        match api::get::<Vec<Category>>("/categories").await {
            Ok(data) => {
                if let Some(first) = data.first() {
                    category_id.set(first.id.clone());
                }
                categories.set(data);
            }
            Err(e) => error_msg.set(Some(e.to_string())),
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let cat_id = category_id.get();
        let amt_str = amount.get();
        let purp = purpose.get().trim().to_string();
        let date = occurred_at.get();
        let n = note.get().trim().to_string();

        if cat_id.is_empty() || purp.is_empty() || date.is_empty() {
            error_msg.set(Some("必須項目を入力してください".to_string()));
            return;
        }
        let amt: i32 = match amt_str.parse() {
            Ok(v) if v > 0 => v,
            _ => {
                error_msg.set(Some("金額は正の整数で入力してください".to_string()));
                return;
            }
        };

        loading.set(true);
        error_msg.set(None);

        // Get file from input
        let document = web_sys::window().unwrap().document().unwrap();
        let file_input = document.get_element_by_id("receipt-input")
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok());
        let file = file_input.as_ref()
            .and_then(|input| input.files())
            .and_then(|files| files.get(0));

        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let form_data = web_sys::FormData::new().unwrap();
            let _ = form_data.append_with_str("category_id", &cat_id);
            let _ = form_data.append_with_str("amount", &amt.to_string());
            let _ = form_data.append_with_str("purpose", &purp);
            let _ = form_data.append_with_str("occurred_at", &date);
            if !n.is_empty() {
                let _ = form_data.append_with_str("note", &n);
            }
            if let Some(f) = file {
                let _ = form_data.append_with_blob_and_filename("receipt", &f, &f.name());
            }

            match api::post_multipart::<Expense>("/expenses", form_data).await {
                Ok(_) => navigate("/expenses", Default::default()),
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
                <h2 class="text-base font-bold text-gray-900 mb-4">"経費申請 新規作成"</h2>
                <div class="card">
                    <form on:submit=on_submit class="space-y-4">
                        <div>
                            <label class="label">"勘定項目 *"</label>
                            <select class="input-field"
                                on:change=move |ev| category_id.set(event_target_value(&ev))>
                                <For
                                    each=move || categories.get()
                                    key=|c| c.id.clone()
                                    children=|cat| {
                                        let id = cat.id.clone();
                                        view! {
                                            <option value={id}>{cat.name.clone()}</option>
                                        }
                                    }
                                />
                            </select>
                        </div>
                        <div>
                            <label class="label">"金額（円） *"</label>
                            <input type="number" class="input-field" placeholder="5000"
                                on:input=move |ev| amount.set(event_target_value(&ev))
                                prop:value=amount />
                        </div>
                        <div>
                            <label class="label">"目的・用途 *"</label>
                            <input type="text" class="input-field" placeholder="クライアント訪問の交通費"
                                on:input=move |ev| purpose.set(event_target_value(&ev))
                                prop:value=purpose />
                        </div>
                        <div>
                            <label class="label">"経費発生日 *"</label>
                            <input type="date" class="input-field"
                                on:input=move |ev| occurred_at.set(event_target_value(&ev))
                                prop:value=occurred_at />
                        </div>
                        <div>
                            <label class="label">"備考・注記"</label>
                            <textarea class="input-field h-20 resize-none" placeholder="補足事項があれば入力"
                                on:input=move |ev| note.set(event_target_value(&ev))
                                prop:value=note />
                        </div>
                        <div>
                            <label class="label">"レシート・領収書"</label>
                            <input id="receipt-input" type="file" accept=".jpg,.jpeg,.png,.pdf"
                                class="block w-full text-sm text-gray-600 file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100" />
                            <p class="text-xs text-gray-400 mt-1">"JPEG / PNG / PDF（最大10MB）"</p>
                        </div>
                        {move || error_msg.get().map(|e| view! { <p class="text-xs text-red-500">{e}</p> })}
                        <div class="flex gap-2 pt-2">
                            <button type="submit" class="btn-primary" disabled=loading>
                                {move || if loading.get() { "送信中..." } else { "申請する" }}
                            </button>
                            <a href="/expenses" class="btn-secondary">"キャンセル"</a>
                        </div>
                    </form>
                </div>
            </div>
        </Layout>
    }
}
