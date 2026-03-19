use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use crate::components::toast::{ToastContainer, use_toast};
use crate::pages::{
    admin_expenses::AdminExpensesPage,
    expenses::{create::ExpenseCreatePage, edit::ExpenseEditPage, list::ExpenseListPage},
    login::LoginPage,
    users::UsersPage,
};
use crate::store::AuthStore;

#[component]
fn AuthGuard(children: Children) -> impl IntoView {
    let auth = use_context::<AuthStore>().expect("AuthStore not provided");
    if auth.is_authenticated() {
        children().into_any()
    } else {
        let window = web_sys::window().unwrap();
        let _ = window.location().set_href("/login");
        view! { <div></div> }.into_any()
    }
}

#[component]
fn AdminGuard(children: Children) -> impl IntoView {
    let auth = use_context::<AuthStore>().expect("AuthStore not provided");
    if auth.is_admin() {
        children().into_any()
    } else {
        let window = web_sys::window().unwrap();
        let _ = window.location().set_href("/expenses");
        view! { <div></div> }.into_any()
    }
}

#[component]
pub fn App() -> impl IntoView {
    let auth = AuthStore::new();
    provide_context(auth.clone());

    let (toast_msgs, show_toast) = use_toast();
    provide_context(show_toast.clone());

    view! {
        <Router>
            <ToastContainer messages=toast_msgs />
            <Routes fallback=|| view! { <div class="p-8 text-gray-500">"ページが見つかりません"</div> }>
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/expenses") view=move || view! {
                    <AuthGuard><ExpenseListPage /></AuthGuard>
                } />
                <Route path=path!("/expenses/new") view=move || view! {
                    <AuthGuard><ExpenseCreatePage /></AuthGuard>
                } />
                <Route path=path!("/expenses/:id/edit") view=move || view! {
                    <AuthGuard><ExpenseEditPage /></AuthGuard>
                } />
                <Route path=path!("/admin/expenses") view=move || view! {
                    <AuthGuard><AdminGuard><AdminExpensesPage /></AdminGuard></AuthGuard>
                } />
                <Route path=path!("/admin/users") view=move || view! {
                    <AuthGuard><AdminGuard><UsersPage /></AdminGuard></AuthGuard>
                } />
                <Route path=path!("/") view=move || {
                    let auth = use_context::<AuthStore>().expect("AuthStore");
                    if auth.is_authenticated() {
                        let window = web_sys::window().unwrap();
                        let _ = window.location().set_href("/expenses");
                        view! { <div></div> }.into_any()
                    } else {
                        let window = web_sys::window().unwrap();
                        let _ = window.location().set_href("/login");
                        view! { <div></div> }.into_any()
                    }
                } />
            </Routes>
        </Router>
    }
}
