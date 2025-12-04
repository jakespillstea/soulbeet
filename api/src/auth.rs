use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub username: String,
    pub user_id: String,
    pub expires_at: i64, // Absolute timestamp (seconds)
}

#[cfg(feature = "server")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
#[cfg(feature = "server")]
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub iat: usize,
    pub exp: usize,
    pub purpose: String, // "access" or "refresh"
}

#[cfg(feature = "server")]
pub fn create_tokens(user_id: String, username: String) -> Result<AuthResponse, String> {
    let secret = env::var("SECRET_KEY").unwrap_or_else(|_| "secret".to_string());
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;

    // Access Token (1 hour)
    let access_exp = now
        .checked_add_signed(chrono::Duration::seconds(3600))
        .expect("valid timestamp")
        .timestamp();

    let access_claims = Claims {
        sub: user_id.clone(),
        username: username.clone(),
        iat,
        exp: access_exp as usize,
        purpose: "access".to_string(),
    };

    let token =
        encode(&Header::default(), &access_claims, &encoding_key).map_err(|e| e.to_string())?;

    // Refresh Token (7 days)
    let refresh_exp = now
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let refresh_claims = Claims {
        sub: user_id.clone(),
        username: username.clone(),
        iat,
        exp: refresh_exp,
        purpose: "refresh".to_string(),
    };

    let refresh_token =
        encode(&Header::default(), &refresh_claims, &encoding_key).map_err(|e| e.to_string())?;

    Ok(AuthResponse {
        token,
        refresh_token,
        username,
        user_id,
        expires_at: access_exp,
    })
}

#[cfg(feature = "server")]
pub fn verify_token(token: &str, expected_purpose: &str) -> Result<Claims, String> {
    let secret = env::var("SECRET_KEY").unwrap_or_else(|_| "secret".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| e.to_string())?;

    if token_data.claims.purpose != expected_purpose {
        return Err("Invalid token purpose".to_string());
    }

    Ok(token_data.claims)
}
