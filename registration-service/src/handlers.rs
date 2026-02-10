
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::json;
use uuid::Uuid;

use axum::response::{IntoResponse, Response};
use crate::models::{CountResult, EventServiceResponse, EventStats, OverviewStats, RegisterRequest, Registration};
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

// ---- QR kod ----

/// GET /registrations/:id/qr - generise QR kod za kartu (poziva Python QR servis)
pub async fn get_ticket_qr(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let user_id = Uuid::parse_str(&user.0.sub).unwrap_or_default();

    let reg = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    let reg = match reg {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Karta ne postoji").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Greska").into_response(),
    };

    if reg.user_id != user_id && user.0.role != "Admin" {
        return (StatusCode::FORBIDDEN, "Nemate pristup").into_response();
    }

    // Pozivamo Python QR servis
    let qr_url = format!("{}/qr", state.qr_service_url);
    let qr_resp = reqwest::Client::new()
        .post(&qr_url)
        .json(&json!({
            "ticket_code": reg.ticket_code,
            "user_email": user.0.email,
        }))
        .send()
        .await;

    match qr_resp {
        Ok(resp) if resp.status().is_success() => {
            let bytes = resp.bytes().await.unwrap_or_default();
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "image/png")],
                bytes.to_vec(),
            )
                .into_response()
        }
        _ => (StatusCode::SERVICE_UNAVAILABLE, "QR servis nije dostupan").into_response(),
    }
}

// ---- Analitike ----

/// GET /analytics/event/:event_id - statistika za jedan dogadjaj
pub async fn analytics_event(
    user: AuthUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<EventStats>>), (StatusCode, String)> {
    if user.0.role != "Organizer" && user.0.role != "Admin" {
        return Err((StatusCode::FORBIDDEN, "Nemate dozvolu".to_string()));
    }

    let stats = sqlx::query_as::<_, EventStats>(
        "SELECT
            event_id,
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'confirmed') as confirmed,
            COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled
         FROM registrations
         WHERE event_id = $1
         GROUP BY event_id",
    )
    .bind(event_id)
    .fetch_optional(&state.db)
    .await;

    match stats {
        Ok(Some(s)) => Ok((StatusCode::OK, Json(ApiResponse::success("Statistika", s)))),
        Ok(None) => Ok((
            StatusCode::OK,
            Json(ApiResponse::success(
                "Nema prijava",
                EventStats {
                    event_id,
                    total: Some(0),
                    confirmed: Some(0),
                    cancelled: Some(0),
                },
            )),
        )),
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        )),
    }
}

/// GET /analytics/overview - ukupna statistika sistema (samo admin)
pub async fn analytics_overview(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<OverviewStats>>), (StatusCode, String)> {
    shared::auth::require_role(&user.0, "Admin")?;

    let total = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM registrations",
    )
    .fetch_one(&state.db).await
    .unwrap_or(CountResult { count: Some(0) }).count.unwrap_or(0);

    let confirmed = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM registrations WHERE status = 'confirmed'",
    )
    .fetch_one(&state.db).await
    .unwrap_or(CountResult { count: Some(0) }).count.unwrap_or(0);

    let unique_events = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(DISTINCT event_id) as count FROM registrations",
    )
    .fetch_one(&state.db).await
    .unwrap_or(CountResult { count: Some(0) }).count.unwrap_or(0);

    let unique_users = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(DISTINCT user_id) as count FROM registrations",
    )
    .fetch_one(&state.db).await
    .unwrap_or(CountResult { count: Some(0) }).count.unwrap_or(0);

    let stats = OverviewStats {
        total_registrations: total,
        total_confirmed: confirmed,
        total_cancelled: total - confirmed,
        unique_events,
        unique_users,
    };

    Ok((StatusCode::OK, Json(ApiResponse::success("Pregled statistike", stats))))
}
