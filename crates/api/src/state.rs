use std::sync::Arc;
use rento_core::auth::AuthService;

// #[derive(Clone)]
pub struct AppState {
    pub db: rento_core::db::Database,
    pub auth: Arc<AuthService>,
    pub jwt_secret: String,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            auth: self.auth.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }
    }
}