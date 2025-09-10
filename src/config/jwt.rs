use anyhow::{Result, anyhow};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

/// Statically get the JWT secret from environment variables
static ENCODING_KEY: OnceCell<EncodingKey> = OnceCell::new();
static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::new();

/// JWT claims struct
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: String, // user_id
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

/// Initializes JWT keys from an environment variable.
/// Must be called once at application startup.
pub fn init_keys() -> Result<()> {
    let secret = env::var("JWT_SECRET").map_err(|_| anyhow!("JWT_SECRET must be set"))?;
    ENCODING_KEY
        .set(EncodingKey::from_secret(secret.as_bytes()))
        .ok();
    DECODING_KEY
        .set(DecodingKey::from_secret(secret.as_bytes()))
        .ok();
    Ok(())
}

#[allow(dead_code)]
/// Creates a new JWT with a user ID, role, and expiration time.
pub fn create_jwt(
    user_id: &str,
    role: &str,
    exp_seconds: usize,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + exp_seconds;

    let claims = JwtClaims {
        id: user_id.to_string(),
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let encoding_key = ENCODING_KEY
        .get()
        .expect("JWT keys not initialized. Call init_keys() first.");

    encode(&Header::default(), &claims, encoding_key)
}

#[allow(dead_code)]
/// Verifies a JWT and returns the claims.
pub fn verify_jwt(token: &str) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error> {
    let decoding_key = DECODING_KEY
        .get()
        .expect("JWT keys not initialized. Call init_keys() first.");

    decode::<JwtClaims>(token, decoding_key, &Validation::default())
}
