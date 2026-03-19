use leptos::prelude::*;
use leptos_router::components::A;
use crate::api;
use crate::components::layout::Layout;
use crate::types::{Expense, ExpenseStatus};

#[component]
pub fn ExpenseListPage() -> impl IntoView {
    let expenses = RwSignal::new(Vec::<Expense>::new());
    let error_msg = RwSignal::new(Option::<String>::None);

    let fetch = move || {
        wasm_bindgen_futures::spawn_local(async move {
            match api::get::<Vec<Expense>>("/expenses").await {
                Ok(data) => expenses.set(data),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };
    fetch();

    view! {
        <Layout>
            <div>
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-base font-bold text-gray-900">"経費申請一覧"</h2>
                    <A href="/expenses/new" attr:class="btn-primary">"＋ 新規申請"</A>
                </div>

                {move || error_msg.get().map(|e| view! { <p class="text-sm text-red-500 mb-3">{e}</p> })}

                <div class="card p-0 overflow-hidden">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-gray-200">
                                <th class="table-header">"目的"</th>
                                <th class="table-header">"金額"</th>
                                <th class="table-header">"発生日"</th>
                                <th class="table-header">"ステータス"</th>
                                <th class="table-header">"添付"</th>
                                <th class="table-header">"操作"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || expenses.get()
                                key=|e| e.id.clone()
                                children=|expense| {
                                    let status_badge = match expense.status {
                                        ExpenseStatus::Pending => "badge-pending",
                                        ExpenseStatus::Approved => "badge-approved",
                                        ExpenseStatus::Rejected => "badge-rejected",
                                    };
                                    let status_label = expense.status.to_string();
                                    let is_pending = matches!(expense.status, ExpenseStatus::Pending);
                                    let edit_href = format!("/expenses/{}/edit", expense.id);
                                    let occurred = expense.occurred_at.get(..10).unwrap_or("").to_string();
                                    view! {
                                        <tr class="table-row">
                                            <td class="table-cell font-medium">{expense.purpose.clone()}</td>
                                            <td class="table-cell">
                                                {format!("¥{}", expense.amount)}
                                            </td>
                                            <td class="table-cell text-gray-500">{occurred}</td>
                                            <td class="table-cell">
                                                <span class={status_badge}>{status_label}</span>
                                            </td>
                                            <td class="table-cell">
                                                {match expense.receipt_file.clone() {
                                                    Some(f) => {
                                                        let view_url = crate::api::upload_url(&f);
                                                        let dl_url = crate::api::upload_url(&f);
                                                        view! {
                                                            <div class="flex gap-2">
                                                                <a href={view_url} target="_blank"
                                                                    class="text-xs text-blue-500 hover:text-blue-700">"表示"</a>
                                                                <a href={dl_url} download={f}
                                                                    class="text-xs text-green-600 hover:text-green-800">"DL"</a>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                    None => view! { <span class="text-xs text-gray-300">"-"</span> }.into_any()
                                                }}
                                            </td>
                                            <td class="table-cell">
                                                {if is_pending {
                                                    view! {
                                                        <A href={edit_href} attr:class="text-xs text-blue-500 hover:text-blue-700">"編集"</A>
                                                    }.into_any()
                                                } else {
                                                    view! { <span class="text-xs text-gray-300">"-"</span> }.into_any()
                                                }}
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
