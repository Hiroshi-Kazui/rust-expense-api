use leptos::prelude::*;
use serde::Serialize;
use crate::api;
use crate::components::layout::Layout;
use crate::types::{User, UserRole};

#[derive(Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
    role: UserRole,
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let users = RwSignal::new(Vec::<User>::new());
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let role = RwSignal::new("User".to_string());
    let error_msg = RwSignal::new(Option::<String>::None);
    let success_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let fetch_users = move || {
        wasm_bindgen_futures::spawn_local(async move {
            match api::get::<Vec<User>>("/users").await {
                Ok(data) => users.set(data),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };
    fetch_users();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let n = name.get().trim().to_string();
        let e = email.get().trim().to_string();
        let p = password.get();
        if n.is_empty() || e.is_empty() || p.is_empty() {
            error_msg.set(Some("全項目を入力してください".to_string()));
            return;
        }
        let r = if role.get() == "Admin" { UserRole::Admin } else { UserRole::User };
        loading.set(true);
        error_msg.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let req = CreateUserRequest { name: n, email: e, password: p, role: r };
            match api::post::<_, User>("/users", &req).await {
                Ok(user) => {
                    users.update(|u| u.push(user));
                    name.set(String::new());
                    email.set(String::new());
                    password.set(String::new());
                    success_msg.set(Some("ユーザーを登録しました".to_string()));
                    loading.set(false);
                }
                Err(err) => {
                    error_msg.set(Some(err.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <Layout>
            <div class="max-w-3xl">
                <h2 class="text-base font-bold text-gray-900 mb-4">"ユーザー管理"</h2>

                // Add form
                <div class="card mb-4">
                    <h3 class="text-sm font-medium text-gray-700 mb-3">"新規ユーザー登録"</h3>
                    <form on:submit=on_submit class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="label">"名前"</label>
                            <input type="text" class="input-field"
                                on:input=move |ev| name.set(event_target_value(&ev))
                                prop:value=name />
                        </div>
                        <div>
                            <label class="label">"メールアドレス"</label>
                            <input type="email" class="input-field"
                                on:input=move |ev| email.set(event_target_value(&ev))
                                prop:value=email />
                        </div>
                        <div>
                            <label class="label">"パスワード"</label>
                            <input type="password" class="input-field"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=password />
                        </div>
                        <div>
                            <label class="label">"ロール"</label>
                            <select class="input-field"
                                on:change=move |ev| role.set(event_target_value(&ev))>
                                <option value="User">"ユーザー"</option>
                                <option value="Admin">"管理者"</option>
                            </select>
                        </div>
                        <div class="col-span-2">
                            {move || error_msg.get().map(|e| view! { <p class="text-xs text-red-500 mb-2">{e}</p> })}
                            {move || success_msg.get().map(|s| view! { <p class="text-xs text-green-600 mb-2">{s}</p> })}
                            <button type="submit" class="btn-primary" disabled=loading>"登録"</button>
                        </div>
                    </form>
                </div>

                // List
                <div class="card p-0 overflow-hidden">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-gray-200">
                                <th class="table-header">"名前"</th>
                                <th class="table-header">"メールアドレス"</th>
                                <th class="table-header">"ロール"</th>
                                <th class="table-header">"登録日"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || users.get()
                                key=|u| u.id.clone()
                                children=|user| {
                                    let role_badge = match user.role {
                                        UserRole::Admin => "badge-approved",
                                        UserRole::User => "badge-pending",
                                    };
                                    let role_label = user.role.to_string();
                                    let created = user.created_at.get(..10).unwrap_or("").to_string();
                                    view! {
                                        <tr class="table-row">
                                            <td class="table-cell font-medium">{user.name.clone()}</td>
                                            <td class="table-cell text-gray-600">{user.email.clone()}</td>
                                            <td class="table-cell">
                                                <span class={role_badge}>{role_label}</span>
                                            </td>
                                            <td class="table-cell text-gray-400">{created}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                    {move || if users.get().is_empty() {
                        view! { <p class="text-sm text-gray-400 text-center py-8">"ユーザーがありません"</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
        </Layout>
    }
}
