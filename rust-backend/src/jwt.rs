use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user id)
    pub username: String, // Username
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
}

pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            expiration_hours: 24 * 7, // 7 days
        }
    }
}

pub fn generate_token(user_id: &str, username: &str, config: &JwtConfig) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(config.expiration_hours))
        .timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .context("Failed to generate JWT token")
}

pub fn verify_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .context("Failed to verify JWT token")?;

    Ok(token_data.claims)
}

pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

