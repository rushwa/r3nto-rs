use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::auth::{get_access_token, get_current_user, clear_tokens};

// Re-export UserResponse from api::auth for convenience
pub use crate::api::auth::UserResponse;

// ============== Auth State ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user: Option<UserInfo>,
    pub is_loading: bool,
}

impl AuthState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// User information stored in auth context
/// Matches backend UserResponse structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
}

// ============== Global Auth Signal ==============

static AUTH: GlobalSignal<AuthState> = GlobalSignal::new(|| AuthState::new());

/// Get the auth signal. Works from ANYWHERE.
pub fn auth_signal() -> Signal<AuthState> {
    AUTH.signal()
}

/// Hook to access auth state. Falls back to global signal if no context provided.
pub fn use_auth() -> Signal<AuthState> {
    try_use_context::<Signal<AuthState>>().unwrap_or_else(|| auth_signal())
}

/// Provide the auth context. Call this ONCE in App before Router.
pub fn provide_auth_context() {
    let sig = auth_signal();
    use_context_provider(|| sig);

    // Start token validation
    spawn(async move {
        let mut sig = auth_signal();
        sig.write().is_loading = true;

        if let Some(_token) = get_access_token() {
            match get_current_user().await {
                Ok(user) => {
                    sig.set(AuthState {
                        is_authenticated: true,
                        user: Some(UserInfo {
                            id: user.id,
                            email: user.email,
                            username: user.username,
                            first_name: user.first_name,
                            last_name: user.last_name,
                            role: user.role,
                            phone_number: user.phone_number,
                            is_active: user.is_active,
                        }),
                        is_loading: false,
                    });
                }
                Err(_) => {
                    clear_tokens();
                    sig.set(AuthState {
                        is_authenticated: false,
                        user: None,
                        is_loading: false,
                    });
                }
            }
        } else {
            sig.set(AuthState {
                is_authenticated: false,
                user: None,
                is_loading: false,
            });
        }
    });
}

/// Set auth state after successful login/register.
/// Uses user data from AuthResponse (which includes UserResponse).
pub fn set_authenticated(user: UserResponse) {
    let mut sig = auth_signal();
    sig.set(AuthState {
        is_authenticated: true,
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            phone_number: user.phone_number,
            is_active: user.is_active,
        }),
        is_loading: false,
    });
}

/// Clear auth state on logout.
pub fn clear_auth() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("access_token");
            let _ = storage.remove_item("refresh_token");
        }
    }

    let mut sig = auth_signal();
    sig.set(AuthState {
        is_authenticated: false,
        user: None,
        is_loading: false,
    });
}
