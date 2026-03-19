use leptos::prelude::*;
use serde::Serialize;
use crate::api;
use crate::components::layout::Layout;
use crate::types::{BulkApproveResponse, Expense, ExpenseStatus};

#[derive(Serialize)]
struct UpdateStatusRequest {
    status: String,
}

#[derive(Serialize)]
struct BulkApproveRequest {
    ids: Vec<String>,
}

#[component]
pub fn AdminExpensesPage() -> impl IntoView {
    let expenses = RwSignal::new(Vec::<Expense>::new());
    let selected_ids = RwSignal::new(Vec::<String>::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let success_msg = RwSignal::new(Option::<String>::None);

    let fetch = move || {
        wasm_bindgen_futures::spawn_local(async move {
            match api::get::<Vec<Expense>>("/expenses").await {
                Ok(data) => expenses.set(data),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };
    fetch();

    let update_status = move |id: String, status: &'static str| {
        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateStatusRequest { status: status.to_string() };
            match api::patch::<_, Expense>(&format!("/expenses/{}/status", id), &req).await {
                Ok(updated) => {
                    expenses.update(|list| {
                        if let Some(e) = list.iter_mut().find(|e| e.id == updated.id) {
                            *e = updated;
                        }
                    });
                    success_msg.set(Some("ステータスを更新しました".to_string()));
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let bulk_approve = move |_: leptos::ev::MouseEvent| {
        let ids = selected_ids.get();
        if ids.is_empty() {
            error_msg.set(Some("申請を選択してください".to_string()));
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            let req = BulkApproveRequest { ids };
            match api::post::<_, BulkApproveResponse>("/expenses/bulk-approve", &req).await {
                Ok(resp) => {
                    selected_ids.set(Vec::new());
                    success_msg.set(Some(format!("{}件を一括承認しました", resp.approved)));
                    // Refresh list
                    match api::get::<Vec<Expense>>("/expenses").await {
                        Ok(data) => expenses.set(data),
                        Err(_) => {}
                    }
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let toggle_select = move |id: String| {
        selected_ids.update(|ids| {
            if let Some(pos) = ids.iter().position(|x| x == &id) {
                ids.remove(pos);
            } else {
                ids.push(id);
            }
        });
    };

    view! {
        <Layout>
            <div>
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-base font-bold text-gray-900">"全申請一覧（管理者）"</h2>
                    <button class="btn-success" on:click=bulk_approve>
                        {move || format!("一括承認（{}件選択中）", selected_ids.get().len())}
                    </button>
                </div>

                {move || error_msg.get().map(|e| view! { <p class="text-sm text-red-500 mb-3">{e}</p> })}
                {move || success_msg.get().map(|s| view! { <p class="text-sm text-green-600 mb-3">{s}</p> })}

                <div class="card p-0 overflow-hidden">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-gray-200">
                                <th class="table-header w-8"></th>
                                <th class="table-header">"目的"</th>
                                <th class="table-header">"金額"</th>
                                <th class="table-header">"発生日"</th>
                                <th class="table-header">"ステータス"</th>
                                <th class="table-header">"操作"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || expenses.get()
                                key=|e| format!("{}-{:?}", e.id, e.status)
                                children=move |expense| {
                                    let status_badge = match expense.status {
                                        ExpenseStatus::Pending => "badge-pending",
                                        ExpenseStatus::Approved => "badge-approved",
                                        ExpenseStatus::Rejected => "badge-rejected",
                                    };
                                    let status_label = expense.status.to_string();
                                    let is_pending = matches!(expense.status, ExpenseStatus::Pending);
                                    let eid = expense.id.clone();
                                    let eid2 = expense.id.clone();
                                    let eid3 = expense.id.clone();
                                    let eid4 = expense.id.clone();
                                    let occurred = expense.occurred_at.get(..10).unwrap_or("").to_string();
                                    let is_selected = move || selected_ids.get().contains(&eid4);
                                    view! {
                                        <tr class="table-row">
                                            <td class="table-cell">
                                                {if is_pending {
                                                    view! {
                                                        <input type="checkbox"
                                                            class="rounded border-gray-300"
                                                            prop:checked=is_selected
                                                            on:change=move |_| toggle_select(eid.clone())
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </td>
                                            <td class="table-cell font-medium">{expense.purpose.clone()}</td>
                                            <td class="table-cell">{format!("¥{}", expense.amount)}</td>
                                            <td class="table-cell text-gray-500">{occurred}</td>
                                            <td class="table-cell">
                                                <span class={status_badge}>{status_label}</span>
                                            </td>
                                            <td class="table-cell">
                                                <div class="flex gap-1">
                                                    {if is_pending {
                                                        let eid2c = eid2.clone();
                                                        let eid3c = eid3.clone();
                                                        view! {
                                                            <button class="text-xs text-green-600 hover:text-green-800 font-medium"
                                                                on:click=move |_| update_status(eid2c.clone(), "Approved")>
                                                                "承認"
                                                            </button>
                                                            <span class="text-gray-300">"/"</span>
                                                            <button class="text-xs text-red-500 hover:text-red-700 font-medium"
                                                                on:click=move |_| update_status(eid3c.clone(), "Rejected")>
                                                                "差し戻し"
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        let eid2c = eid2.clone();
                                                        view! {
                                                            <button class="text-xs text-gray-400 hover:text-gray-600"
                                                                on:click=move |_| update_status(eid2c.clone(), "Pending")>
                                                                "戻す"
                                                            </button>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                    {move || if expenses.get().is_empty() {
                        view! { <p class="text-sm text-gray-400 text-center py-8">"申請がありません"</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
        </Layout>
    }
}
