use leptos::prelude::*;
use leptos_router::components::A;
use crate::store::AuthStore;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let auth = use_context::<AuthStore>().expect("AuthStore not provided");
    let auth_admin = auth.clone();
    let auth_name = auth.clone();
    let is_admin = move || auth_admin.is_admin();
    let user_name = move || auth_name.user.get().map(|u| u.name).unwrap_or_default();

    view! {
        <div class="flex h-screen">
            // Sidebar
            <aside class="w-52 bg-white border-r border-gray-200 flex flex-col">
                <div class="px-4 py-4 border-b border-gray-200">
                    <h1 class="text-sm font-bold text-gray-900">"経費管理"</h1>
                    <p class="text-xs text-gray-500 mt-0.5">{user_name}</p>
                </div>
                <nav class="flex-1 px-2 py-3 space-y-1">
                    <A href="/expenses" attr:class="nav-link">
                        "📋 経費申請"
                    </A>
                    <A href="/expenses/new" attr:class="nav-link">
                        "＋ 新規申請"
                    </A>
                    {move || if is_admin() {
                        view! {
                            <div class="pt-3">
                                <p class="px-2 text-xs text-gray-400 font-medium mb-1">"管理"</p>
                                <A href="/admin/expenses" attr:class="nav-link">"📊 全申請一覧"</A>
                                <A href="/admin/users" attr:class="nav-link">"👥 ユーザー"</A>
                                <A href="/admin/categories" attr:class="nav-link">"🏷 勘定項目"</A>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </nav>
                <div class="px-3 py-3 border-t border-gray-200">
                    <button
                        class="w-full text-left text-xs text-gray-500 hover:text-gray-700 px-2 py-1"
                        on:click=move |_| {
                            auth.logout();
                            let window = web_sys::window().unwrap();
                            let _ = window.location().set_href("/login");
                        }
                    >
                        "ログアウト"
                    </button>
                </div>
            </aside>
            // Main content
            <main class="flex-1 overflow-y-auto p-6">
                {children()}
            </main>
        </div>
    }
}
