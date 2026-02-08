// Handleri za auth rute (register, login, health)

use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use serde_json::json;

use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User};
use crate::AppState;
use shared::auth::create_token;
use shared::models::ApiResponse;

/// GET /health - da li servis radi
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "auth-service",
        "status": "ok"
    }))
}

/// POST /register - registracija novog korisnika
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<AuthResponse>>) {
    // Hesiramo lozinku pre cuvanja u bazu
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

    // Upisujemo korisnika u bazu
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
            // Pravimo JWT token za novog korisnika
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

/// POST /login - prijava korisnika
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> (StatusCode, Json<ApiResponse<AuthResponse>>) {
    // Trazimo korisnika po email-u
    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(user)) => {
            // Proveravamo da li se lozinka poklapa
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
