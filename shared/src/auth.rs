// JWT autentifikacija

use axum::http::{header, HeaderMap, StatusCode};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Podaci koji se cuvaju unutar JWT tokena
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

/// Pravi novi JWT token za korisnika (traje 24h)
pub fn create_token(
    user_id: &str,
    email: &str,
    role: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: now + 24 * 3600,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Cita i proverava JWT token
pub fn validate_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Izvlaci Claims iz Authorization headera
/// Koristi se u handlerima: let claims = extract_claims(&headers, &state.jwt_secret)?;
pub fn extract_claims(
    headers: &HeaderMap,
    jwt_secret: &str,
) -> Result<Claims, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Nedostaje Authorization header".to_string(),
        ))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or((
        StatusCode::UNAUTHORIZED,
        "Format mora biti: Bearer <token>".to_string(),
    ))?;

    validate_token(token, jwt_secret).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "Token je istekao ili nije validan".to_string(),
        )
    })
}

/// Proverava da li korisnik ima odredjenu ulogu. Admin uvek prolazi.
pub fn require_role(claims: &Claims, required: &str) -> Result<(), (StatusCode, String)> {
    if claims.role == required || claims.role == "Admin" {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            format!("Potrebna uloga: {}", required),
        ))
    }
}
