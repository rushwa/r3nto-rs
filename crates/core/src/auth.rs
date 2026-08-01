// crates/core/src/auth.rs

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::models::{TokenClaims, UserRole};
use crate::error::Result;

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

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::error::RentoError::PasswordHash(format!("{}", e)))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| crate::error::RentoError::PasswordHash(format!("{}", e)))?;
        Ok(self.argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    pub fn generate_tokens(&self, user_id: Uuid, role: &UserRole, username: &str, email: &str) -> Result<(String, String)> {
        let now = Utc::now();
        let access_exp = now + Duration::minutes(60);
        let refresh_exp = now + Duration::days(7);

        let access_claims = TokenClaims {
            sub: user_id,
            role: role.clone(),
            username: username.to_string(),
            email: email.to_string(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
        };

        let refresh_claims = TokenClaims {
            sub: user_id,
            role: role.clone(),
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

    pub fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        let validation = Validation::default();
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;
        Ok(token_data.claims)
    }

    pub fn generate_verification_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    }
}
