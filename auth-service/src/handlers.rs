// Handleri za auth rute

use axum::{extract::State, http::HeaderMap, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;
use uuid::Uuid;

use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User};
use crate::AppState;
use shared::auth::{create_token, extract_claims};
use shared::models::ApiResponse;

/// GET /health
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "auth-service",
        "status": "ok"
    }))
}

/// me - returna podatke o logovanom korisniku

pub async fn me(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<AuthResponse>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };

    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&claims.sub).unwrap_or_default())
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(db_user)) => {
            let response = AuthResponse {
                token: String::new(),
                user_id: db_user.id.to_string(),
                email: db_user.email,
                role: db_user.role,
            };
            (StatusCode::OK, Json(ApiResponse::success("Korisnik pronadjen", response)))
        }
        _ => (StatusCode::NOT_FOUND, Json(ApiResponse::error("Korisnik ne postoji u bazi"))),
    }
}

/// POST /register
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<AuthResponse>>) {
    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Greska pri hesiranju lozinke")),
            );
        }
    };

    let id = Uuid::new_v4();

    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind("User")
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => {
            let token = create_token(
                &user.id.to_string(),
                &user.email,
                &user.role,
                &state.jwt_secret,
            )
            .unwrap_or_default();

            let response = AuthResponse {
                token,
                user_id: user.id.to_string(),
                email: user.email,
                role: user.role,
            };
            (
                StatusCode::CREATED,
                Json(ApiResponse::success("Registracija uspesna", response)),
            )
        }
        Err(e) => {
            let msg = format!("Greska pri registraciji: {}", e);
            (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&msg)))
        }
    }
}

/// POST /login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> (StatusCode, Json<ApiResponse<AuthResponse>>) {
    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(user)) => {
            if verify(&req.password, &user.password_hash).unwrap_or(false) {
                let token = create_token(
                    &user.id.to_string(),
                    &user.email,
                    &user.role,
                    &state.jwt_secret,
                )
                .unwrap_or_default();

                let response = AuthResponse {
                    token,
                    user_id: user.id.to_string(),
                    email: user.email,
                    role: user.role,
                };
                (
                    StatusCode::OK,
                    Json(ApiResponse::success("Login uspesan", response)),
                )
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::error("Pogresna lozinka")),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Korisnik sa tim emailom ne postoji")),
        ),
        Err(e) => {
            let msg = format!("Greska pri loginu: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&msg)),
            )
        }
    }
}
