
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::json;
use uuid::Uuid;

use crate::models::{CountResult, EventServiceResponse, RegisterRequest, Registration};
use crate::AppState;
use shared::auth::AuthUser;
use shared::models::ApiResponse;

/// GET health
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "registration-service",
        "status": "ok"
    }))
}

/// registration for event
pub async fn register_for_event(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<Registration>>) {
    let user_id = Uuid::parse_str(&user.0.sub).unwrap_or_default();

    // jel user vec prijavljen
    let existing = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE event_id = $1 AND user_id = $2 AND status = 'confirmed'",
    )
    .bind(req.event_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some(_)) = existing {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error("Vec ste prijavljeni na ovaj dogadjaj")),
        );
    }

    // pitamo servis za kapacitet
    let event_url = format!("{}/events/{}", state.event_service_url, req.event_id);
    let event_resp = reqwest::get(&event_url).await;

    let event_data = match event_resp {
        Ok(resp) => match resp.json::<EventServiceResponse>().await {
            Ok(data) if data.success && data.data.is_some() => data.data.unwrap(),
            _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("Dogadjaj ne postoji"))),
        },
        Err(_) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse::error("Event servis nije dostupan")),
        ),
    };

    // count registrations
    let count = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM registrations WHERE event_id = $1 AND status = 'confirmed'",
    )
    .bind(req.event_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(CountResult { count: Some(0) });

    let current_count = count.count.unwrap_or(0);

    if current_count >= event_data.capacity as i64 {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error("Dogadjaj je popunjen, nema slobodnih mesta")),
        );
    }

    // Generisemo ticket kod
    let ticket_code = format!("TKT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

    // Write registration
    let result = sqlx::query_as::<_, Registration>(
        "INSERT INTO registrations (id, event_id, user_id, ticket_code, status)
         VALUES (gen_random_uuid(), $1, $2, $3, 'confirmed')
         RETURNING *",
    )
    .bind(req.event_id)
    .bind(user_id)
    .bind(&ticket_code)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(reg) => (
            StatusCode::CREATED,
            Json(ApiResponse::success("Uspesno prijavljeni", reg)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// Cancel registration
pub async fn cancel_registration(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Registration>>) {
    let user_id = Uuid::parse_str(&user.0.sub).unwrap_or_default();

    // proveri jel postoji i pripada useru
    let existing = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match existing {
        Ok(Some(reg)) => {
            if reg.user_id != user_id && user.0.role != "Admin" {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Ne mozete otkazati tudju prijavu")),
                );
            }

            let result = sqlx::query_as::<_, Registration>(
                "UPDATE registrations SET status = 'cancelled' WHERE id = $1 RETURNING *",
            )
            .bind(id)
            .fetch_one(&state.db)
            .await;

            match result {
                Ok(cancelled) => (
                    StatusCode::OK,
                    Json(ApiResponse::success("Prijava otkazana", cancelled)),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Greska: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Prijava ne postoji")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// get my registrations
pub async fn my_registrations(
    user: AuthUser,
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<Vec<Registration>>>) {
    let user_id = Uuid::parse_str(&user.0.sub).unwrap_or_default();

    let result = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(regs) => (
            StatusCode::OK,
            Json(ApiResponse::success("Moje prijave", regs)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// get all registrations for event
pub async fn event_registrations(
    user: AuthUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<Registration>>>) {
    // organizer i admin mogu da vide listu registracija
    if user.0.role != "Organizer" && user.0.role != "Admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Nemate dozvolu")),
        );
    }

    let result = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE event_id = $1 ORDER BY created_at",
    )
    .bind(event_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(regs) => (
            StatusCode::OK,
            Json(ApiResponse::success("Prijave za dogadjaj", regs)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// GET registration ticket
pub async fn get_ticket(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Registration>>) {
    let user_id = Uuid::parse_str(&user.0.sub).unwrap_or_default();

    let result = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(reg)) => {
            if reg.user_id != user_id && user.0.role != "Admin" {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Nemate pristup ovoj karti")),
                );
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success("Karta", reg)),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Karta ne postoji")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}
