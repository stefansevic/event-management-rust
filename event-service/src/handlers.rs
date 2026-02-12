// Handleri za event servis

use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde_json::json;
use uuid::Uuid;

use crate::models::{CreateEventRequest, Event, EventQuery, UpdateEventRequest};
use crate::AppState;
use shared::auth::{extract_claims, require_role};
use shared::models::ApiResponse;

/// GET health
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "event-service",
        "status": "ok"
    }))
}

/// create event
pub async fn create_event(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Event>>), (StatusCode, String)> {
    let claims = extract_claims(&headers, &state.jwt_secret)?;
    require_role(&claims, "Organizer")?;

    let organizer_id = Uuid::parse_str(&claims.sub).unwrap_or_default();

    // ne moze dogadjaj u proslosti
    if req.date_time < chrono::Utc::now().naive_utc() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Datum dogadjaja ne moze biti u proslosti")),
        ));
    }

    let result = sqlx::query_as::<_, Event>(
        "INSERT INTO events (id, organizer_id, title, description, location, date_time, capacity, category)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7)
         RETURNING *",
    )
    .bind(organizer_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.location)
    .bind(req.date_time)
    .bind(req.capacity)
    .bind(&req.category)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(event) => Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success("Dogadjaj kreiran", event)),
        )),
        Err(e) => Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        )),
    }
}

/// get events
pub async fn list_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<Vec<Event>>>) {
    // u zavisnosti od filtera  
    let result = match (&params.category, &params.search) {
        (Some(cat), Some(search)) => {
            let pattern = format!("%{}%", search);
            sqlx::query_as::<_, Event>(
                "SELECT * FROM events WHERE category = $1 AND (title ILIKE $2 OR description ILIKE $2) ORDER BY date_time",
            )
            .bind(cat)
            .bind(&pattern)
            .fetch_all(&state.db)
            .await
        }
        (Some(cat), None) => {
            sqlx::query_as::<_, Event>(
                "SELECT * FROM events WHERE category = $1 ORDER BY date_time",
            )
            .bind(cat)
            .fetch_all(&state.db)
            .await
        }
        (None, Some(search)) => {
            let pattern = format!("%{}%", search);
            sqlx::query_as::<_, Event>(
                "SELECT * FROM events WHERE title ILIKE $1 OR description ILIKE $1 ORDER BY date_time",
            )
            .bind(&pattern)
            .fetch_all(&state.db)
            .await
        }
        (None, None) => {
            sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY date_time")
                .fetch_all(&state.db)
                .await
        }
    };

    match result {
        Ok(events) => (
            StatusCode::OK,
            Json(ApiResponse::success("Lista dogadjaja", events)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// get event by id
pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Event>>) {
    let result = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(event)) => (
            StatusCode::OK,
            Json(ApiResponse::success("Dogadjaj pronadjen", event)),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Dogadjaj ne postoji")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// update event
pub async fn update_event(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEventRequest>,
) -> (StatusCode, Json<ApiResponse<Event>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };

    // dal postoji
    let existing = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    let event = match existing {
        Ok(Some(e)) => e,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("Dogadjaj ne postoji"))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&format!("Greska: {}", e)))),
    };

    // DAC access
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();
    if event.organizer_id != user_id && claims.role != "Admin" {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("Nemate dozvolu da menjate ovaj dogadjaj")));
    }

    // update poslata polja
    let result = sqlx::query_as::<_, Event>(
        "UPDATE events SET
            title = COALESCE($2, title),
            description = COALESCE($3, description),
            location = COALESCE($4, location),
            date_time = COALESCE($5, date_time),
            capacity = COALESCE($6, capacity),
            category = COALESCE($7, category),
            updated_at = NOW()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.location)
    .bind(req.date_time)
    .bind(req.capacity)
    .bind(&req.category)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(updated) => (
            StatusCode::OK,
            Json(ApiResponse::success("Dogadjaj azuriran", updated)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Greska: {}", e))),
        ),
    }
}

/// delete event
pub async fn delete_event(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let claims = match extract_claims(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err((status, msg)) => return (status, Json(ApiResponse::error(&msg))),
    };

    let existing = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    let event = match existing {
        Ok(Some(e)) => e,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("Dogadjaj ne postoji"))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&format!("Greska: {}", e)))),
    };

    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();
    if event.organizer_id != user_id && claims.role != "Admin" {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("Nemate dozvolu da obrisete ovaj dogadjaj")));
    }

    let _ = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await;

    (StatusCode::OK, Json(ApiResponse::success("Dogadjaj obrisan", "ok".to_string())))
}
