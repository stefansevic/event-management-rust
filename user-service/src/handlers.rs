// Handleri za user servis

use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use serde_json::json;
use uuid::Uuid;

use crate::models::{ProfileRequest, UserProfile};
use crate::AppState;
use shared::auth::{extract_claims, require_role};
use shared::models::ApiResponse;

/// GET /health
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "user-service",
        "status": "ok"
    }))
}

/// GET /profile 
pub async fn get_my_profile(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<UserProfile>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

    let result = sqlx::query_as::<_, UserProfile>(
        "SELECT * FROM user_profiles WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(profile)) => (
            StatusCode::OK,
            Json(ApiResponse::success("Profil pronadjen", profile)),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Profil ne postoji, kreiraj ga sa PUT /profile")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// PUT /profile - create/update profile
pub async fn upsert_profile(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<ProfileRequest>,
) -> (StatusCode, Json<ApiResponse<UserProfile>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

    // create ako ne postoji, update ako postoji
    let result = sqlx::query_as::<_, UserProfile>(
        "INSERT INTO user_profiles (id, user_id, full_name, phone, bio)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         ON CONFLICT (user_id) DO UPDATE
         SET full_name = $2, phone = $3, bio = $4, updated_at = NOW()
         RETURNING *",
    )
    .bind(user_id)
    .bind(&req.full_name)
    .bind(&req.phone)
    .bind(&req.bio)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(profile) => (
            StatusCode::OK,
            Json(ApiResponse::success("Profil sacuvan", profile)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// GET /profiles
pub async fn list_profiles(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<UserProfile>>>), (StatusCode, String)> {
    let claims = extract_claims(&headers, &state.jwt_secret)?;
    require_role(&claims, "Admin")?;

    let profiles = sqlx::query_as::<_, UserProfile>("SELECT * FROM user_profiles")
        .fetch_all(&state.db)
        .await;

    match profiles {
        Ok(list) => Ok((
            StatusCode::OK,
            Json(ApiResponse::success("Lista profila", list)),
        )),
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        )),
    }
}

/// GET /profiles po id
pub async fn get_profile_by_id(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<UserProfile>>) {
    let _claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let result = sqlx::query_as::<_, UserProfile>(
        "SELECT * FROM user_profiles WHERE user_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(profile)) => (
            StatusCode::OK,
            Json(ApiResponse::success("Profil pronadjen", profile)),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Profil ne postoji")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// DELETE /profiles
pub async fn delete_profile(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    let claims = extract_claims(&headers, &state.jwt_secret)?;
    require_role(&claims, "Admin")?;

    let result = sqlx::query("DELETE FROM user_profiles WHERE user_id = $1")
        .bind(id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Ok((
            StatusCode::OK,
            Json(ApiResponse::success("Profil obrisan", "ok".to_string())),
        )),
        Ok(_) => Ok((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Profil ne postoji")),
        )),
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        )),
    }
}
