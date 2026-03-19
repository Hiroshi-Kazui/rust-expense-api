use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::Serialize;
use crate::api;
use crate::store::AuthStore;
use crate::types::LoginResponse;

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_context::<AuthStore>().expect("AuthStore not provided");
    let navigate = use_navigate();

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let email_val = email.get();
        let password_val = password.get();
        if email_val.is_empty() || password_val.is_empty() {
            error_msg.set(Some("メールアドレスとパスワードを入力してください".to_string()));
            return;
        }
        loading.set(true);
        error_msg.set(None);
        let auth = auth.clone();
        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let req = LoginRequest { email: email_val, password: password_val };
            match api::post_no_auth::<_, LoginResponse>("/auth/login", &req).await {
                Ok(resp) => {
                    auth.login(resp.token, resp.user);
                    navigate("/expenses", Default::default());
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gray-50 flex items-center justify-center">
            <div class="w-full max-w-sm">
                <div class="card">
                    <h2 class="text-lg font-bold text-gray-900 mb-6">"経費管理システム"</h2>
                    <form on:submit=on_submit class="space-y-4">
                        <div>
                            <label class="label">"メールアドレス"</label>
                            <input
                                type="email"
                                class="input-field"
                                placeholder="admin@example.com"
                                on:input=move |ev| email.set(event_target_value(&ev))
                                prop:value=email
                            />
                        </div>
                        <div>
                            <label class="label">"パスワード"</label>
                            <input
                                type="password"
                                class="input-field"
                                placeholder="••••••••"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=password
                            />
                        </div>
                        {move || error_msg.get().map(|e| view! {
                            <p class="text-xs text-red-500">{e}</p>
                        })}
                        <button
                            type="submit"
                            class="btn-primary w-full"
                            disabled=loading
                        >
                            {move || if loading.get() { "ログイン中..." } else { "ログイン" }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
