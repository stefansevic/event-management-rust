use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use serde_json::json;
use uuid::Uuid;

use axum::response::{IntoResponse, Response};
use crate::models::{CountResult, EventServiceResponse, RegisterRequest, Registration};
use crate::AppState;
use shared::auth::extract_claims;
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
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<Registration>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

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

    // pitamo event servis za kapacitet
    let event_url = format!("{}/events/{}", state.event_service_url, req.event_id);
    tracing::info!("Pozivam event servis: {}", event_url);

    let event_data = match reqwest::get(&event_url).await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::info!("Event servis odgovorio: status={}, body={}", status, &body);

            match serde_json::from_str::<EventServiceResponse>(&body) {
                Ok(data) if data.success && data.data.is_some() => data.data.unwrap(),
                Ok(data) => {
                    tracing::warn!("Event servis vratio success=false: {:?}", data);
                    return (StatusCode::NOT_FOUND, Json(ApiResponse::error("Dogadjaj ne postoji")));
                }
                Err(e) => {
                    tracing::error!("Greska pri parsiranju odgovora event servisa: {}", e);
                    return (StatusCode::NOT_FOUND, Json(ApiResponse::error("Dogadjaj ne postoji")));
                }
            }
        }
        Err(e) => {
            tracing::error!("Ne mogu da kontaktiram event servis: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::error("Event servis nije dostupan")),
            );
        }
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
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Registration>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

    // proveri jel postoji i pripada useru
    let existing = sqlx::query_as::<_, Registration>(
        "SELECT * FROM registrations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match existing {
        Ok(Some(reg)) => {
            if reg.user_id != user_id && claims.role != "Admin" {
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

/// Internal: otkazuje sve prijave za dogadjaj (poziva event-service pri brisanju dogadjaja)
pub async fn cancel_registrations_for_event(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let result = sqlx::query("UPDATE registrations SET status = 'cancelled' WHERE event_id = $1")
        .bind(event_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(rows) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Prijave otkazane",
                json!({ "updated": rows.rows_affected() }),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// get my registrations
pub async fn my_registrations(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<Vec<Registration>>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

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

// ---- QR kod ----

/// GET /registrations/:id/qr - generise QR kod za kartu (poziva Python QR servis)
pub async fn get_ticket_qr(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, msg).into_response(),
    };
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

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

    if reg.user_id != user_id && claims.role != "Admin" {
        return (StatusCode::FORBIDDEN, "Nemate pristup").into_response();
    }

    // Pozivamo Python QR servis
    let qr_url = format!("{}/qr", state.qr_service_url);
    let qr_resp = reqwest::Client::new()
        .post(&qr_url)
        .json(&json!({
            "ticket_code": reg.ticket_code,
            "user_email": claims.email,
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
