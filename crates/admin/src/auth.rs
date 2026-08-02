use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub is_auth: bool,
    pub user_role: Option<String>,
    pub user_name: Option<String>,
}

impl AuthContext {
    pub fn is_authenticated(&self) -> bool {
        self.is_auth
    }
    
    pub fn role(&self) -> Option<String> {
        self.user_role.clone()
    }
    
    pub fn user_name(&self) -> String {
        self.user_name.clone().unwrap_or_else(|| "User".to_string())
    }
    
    pub fn logout(&self) {
        // TODO: Clear token from localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(storage) = window.local_storage() {
                if let Some(s) = storage {
                    let _ = s.remove_item("access_token");
                }
            }
        }
    }
}

pub fn use_auth_context() -> AuthContext {
    use_context()
}
