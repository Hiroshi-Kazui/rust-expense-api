use leptos::prelude::*;
use serde::Serialize;
use crate::api;
use crate::components::layout::Layout;
use crate::types::Category;

#[derive(Serialize)]
struct CreateCategoryRequest {
    name: String,
}

#[component]
pub fn CategoriesPage() -> impl IntoView {
    let categories = RwSignal::new(Vec::<Category>::new());
    let new_name = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let success_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let fetch_categories = move || {
        wasm_bindgen_futures::spawn_local(async move {
            match api::get::<Vec<Category>>("/categories").await {
                Ok(data) => categories.set(data),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    fetch_categories();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get().trim().to_string();
        if name.is_empty() {
            error_msg.set(Some("名前を入力してください".to_string()));
            return;
        }
        loading.set(true);
        error_msg.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, Category>("/categories", &CreateCategoryRequest { name }).await {
                Ok(cat) => {
                    categories.update(|c| c.push(cat));
                    new_name.set(String::new());
                    success_msg.set(Some("勘定項目を追加しました".to_string()));
                    loading.set(false);
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(3000).await;
                        success_msg.set(None);
                    });
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
            <div class="max-w-2xl">
                <h2 class="text-base font-bold text-gray-900 mb-4">"勘定項目"</h2>

                // Add form
                <div class="card mb-4">
                    <h3 class="text-sm font-medium text-gray-700 mb-3">"新規追加"</h3>
                    <form on:submit=on_submit class="flex gap-2">
                        <input
                            type="text"
                            class="input-field flex-1"
                            placeholder="勘定項目名"
                            on:input=move |ev| new_name.set(event_target_value(&ev))
                            prop:value=new_name
                        />
                        <button type="submit" class="btn-primary" disabled=loading>
                            "追加"
                        </button>
                    </form>
                    {move || error_msg.get().map(|e| view! { <p class="text-xs text-red-500 mt-2">{e}</p> })}
                    {move || success_msg.get().map(|s| view! { <p class="text-xs text-green-600 mt-2">{s}</p> })}
                </div>

                // List
                <div class="card p-0 overflow-hidden">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-gray-200">
                                <th class="table-header">"名前"</th>
                                <th class="table-header">"ID"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || categories.get()
                                key=|c| c.id.clone()
                                children=|cat| view! {
                                    <tr class="table-row">
                                        <td class="table-cell font-medium">{cat.name.clone()}</td>
                                        <td class="table-cell text-gray-400 text-xs font-mono">{cat.id.clone()}</td>
                                    </tr>
                                }
                            />
                        </tbody>
                    </table>
                    {move || if categories.get().is_empty() {
                        view! { <p class="text-sm text-gray-400 text-center py-8">"勘定項目がありません"</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
        </Layout>
    }
}
