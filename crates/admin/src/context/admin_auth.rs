use dioxus::prelude::*;
use serde::Deserialize;

#[derive(Clone, Default, Debug)]
pub struct AdminAuthState {
    pub token: Option<String>,
    pub user: Option<AdminUser>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

pub fn use_admin_auth() -> Signal<AdminAuthState> {
    use_context::<Signal<AdminAuthState>>()
}

fn get_window_storage() -> Option<web_sys::Storage> {
    let window = web_sys::window()?;
    window.local_storage().ok()?
}

pub fn get_token() -> Option<String> {
    get_window_storage()?.get_item("admin_token").ok()?
}

pub fn set_token(token: &str) {
    if let Some(s) = get_window_storage() {
        let _ = s.set_item("admin_token", token);
    }
}

pub fn clear_token() {
    if let Some(s) = get_window_storage() {
        let _ = s.remove_item("admin_token");
    }
}
