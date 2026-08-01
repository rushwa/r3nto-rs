// crates/web/src/hooks/mod.rs
use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone_number: Option<String>,
    pub identification_no: Option<String>,
    pub county: Option<String>,
    pub constituency: Option<String>,
    pub ward: Option<String>,
    pub location: Option<String>,
    pub phone_verified: bool,
    pub subscribed: bool,
    pub is_active: bool,
}

pub fn use_auth() -> AuthState {
    let user = use_signal(|| None::<UserResponse>);

    AuthState { user }
}

pub struct AuthState {
    user: Signal<Option<UserResponse>>,
}

impl AuthState {
    pub fn user(&self) -> Option<UserResponse> {
        // Read the signal value, not call ourselves
        self.user.read().clone()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.read().is_some()
    }
}
