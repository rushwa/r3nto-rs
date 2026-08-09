use std::sync::Arc;
use rento_core::auth::AuthService;
use rento_core::email::EmailService;
use crate::services::mpesa::MpesaClient;

pub struct AppState {
    pub db: rento_core::db::Database,
    pub auth: Arc<AuthService>,
    pub jwt_secret: String,
    pub email: Arc<EmailService>,
    pub mpesa: Arc<MpesaClient>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            auth: self.auth.clone(),
            jwt_secret: self.jwt_secret.clone(),
            email: self.email.clone(),
            mpesa: self.mpesa.clone(),
        }
    }
}