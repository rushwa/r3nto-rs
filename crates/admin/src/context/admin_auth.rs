use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const TOKEN_KEY: &str = "rento_admin_token";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub is_superuser: bool,
    #[serde(default)]
    pub is_staff: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AdminAuthState {
    pub token: Option<String>,
    pub user: Option<AdminUser>,
}

pub fn use_admin_auth() -> Signal<AdminAuthState> {
    use_context()
}

pub fn get_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .and_then(|s| s.and_then(|s| s.get_item(TOKEN_KEY).ok()))
            .flatten()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

pub fn set_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(TOKEN_KEY, token);
            }
        }
    }
}

pub fn clear_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(TOKEN_KEY);
            }
        }
    }
}
