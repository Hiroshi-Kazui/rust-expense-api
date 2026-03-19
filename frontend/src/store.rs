use leptos::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::types::User;

pub const TOKEN_KEY: &str = "expense_token";
pub const USER_KEY: &str = "expense_user";

#[derive(Clone, Debug)]
pub struct AuthStore {
    pub token: RwSignal<Option<String>>,
    pub user: RwSignal<Option<User>>,
}

impl AuthStore {
    pub fn new() -> Self {
        let token = LocalStorage::get::<String>(TOKEN_KEY).ok();
        let user = LocalStorage::get::<User>(USER_KEY).ok();
        Self {
            token: RwSignal::new(token),
            user: RwSignal::new(user),
        }
    }

    pub fn login(&self, token: String, user: User) {
        let _ = LocalStorage::set(TOKEN_KEY, &token);
        let _ = LocalStorage::set(USER_KEY, &user);
        self.token.set(Some(token));
        self.user.set(Some(user));
    }

    pub fn logout(&self) {
        LocalStorage::delete(TOKEN_KEY);
        LocalStorage::delete(USER_KEY);
        self.token.set(None);
        self.user.set(None);
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.get().is_some()
    }

    pub fn is_admin(&self) -> bool {
        self.user.get().map(|u| matches!(u.role, crate::types::UserRole::Admin)).unwrap_or(false)
    }
}
