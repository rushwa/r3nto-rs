// crates/core/src/auth.rs
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::error::Result;
use crate::models::TokenClaims;

// ───────────────────────────────────────────
// AuthService — handles password hashing and JWT tokens
// ───────────────────────────────────────────
pub struct AuthService {
    jwt_secret: String,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            argon2: Argon2::default(),
        }
    }

    // ───────────────────────────────────────────
    // Password Hashing
    // ───────────────────────────────────────────

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::error::RentoError::PasswordHash(format!("{}", e)))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| crate::error::RentoError::PasswordHash(format!("{}", e)))?;
        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    // ───────────────────────────────────────────
    // JWT Token Generation
    // ✅ FIX: role is now &str (not &UserRole)
    // This allows "PROPERTY_OWNER", "AGENT", "ADMIN", etc.
    // to be stored directly in the JWT without enum mismatch errors.
    // ───────────────────────────────────────────
    pub fn generate_tokens(
        &self,
        user_id: Uuid,
        role: &str,
        username: &str,
        email: &str,
    ) -> Result<(String, String)> {
        let now = Utc::now();
        let access_exp = now + Duration::minutes(60);
        let refresh_exp = now + Duration::days(7);

        let access_claims = TokenClaims {
            sub: user_id,
            role: role.to_string(), // ✅ String now
            username: username.to_string(),
            email: email.to_string(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
        };

        let refresh_claims = TokenClaims {
            sub: user_id,
            role: role.to_string(), // ✅ String now
            username: username.to_string(),
            email: email.to_string(),
            exp: refresh_exp.timestamp(),
            iat: now.timestamp(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        Ok((access_token, refresh_token))
    }

    // ───────────────────────────────────────────
    // JWT Token Verification
    // ───────────────────────────────────────────
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        let validation = Validation::default();
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;
        Ok(token_data.claims)
    }

    // ───────────────────────────────────────────
    // Utility: Generate 6-digit verification code
    // ───────────────────────────────────────────
    pub fn generate_verification_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    }
}