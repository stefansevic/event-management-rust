
use axum::{
    extract::{Path, Request, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use serde_json::json;

use crate::proxy::forward_request;
use crate::AppState;

// citanje body-ja

async fn read_body(req: Request) -> (HeaderMap, String) {
    let headers = req.headers().clone();
    let body_bytes = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .unwrap_or_default();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    (headers, body)
}


/// GET health - proverava sve servise
pub async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let client = &state.client;

    let auth_ok = client.get(format!("{}/health", state.auth_url)).send().await.is_ok();
    let user_ok = client.get(format!("{}/health", state.user_url)).send().await.is_ok();
    let event_ok = client.get(format!("{}/health", state.event_url)).send().await.is_ok();
    let reg_ok = client.get(format!("{}/health", state.registration_url)).send().await.is_ok();

    Json(json!({
        "service": "api-gateway",
        "status": "ok",
        "services": {
            "auth": if auth_ok { "up" } else { "down" },
            "user": if user_ok { "up" } else { "down" },
            "event": if event_ok { "up" } else { "down" },
            "registration": if reg_ok { "up" } else { "down" },
        }
    }))
}

//  Auth rute 

pub async fn auth_register(State(state): State<AppState>, req: Request) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/register", state.auth_url);
    forward_request(&state.client, "POST", &url, &headers, Some(body)).await
}

pub async fn auth_login(State(state): State<AppState>, req: Request) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/login", state.auth_url);
    forward_request(&state.client, "POST", &url, &headers, Some(body)).await
}

pub async fn auth_me(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/me", state.auth_url);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

//  User rute 

pub async fn user_profile_get(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/profile", state.user_url);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn user_profile_put(State(state): State<AppState>, req: Request) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/profile", state.user_url);
    forward_request(&state.client, "PUT", &url, &headers, Some(body)).await
}

pub async fn user_profiles_list(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/profiles", state.user_url);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn user_profile_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/profiles/{}", state.user_url, id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn user_profile_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/profiles/{}", state.user_url, id);
    forward_request(&state.client, "DELETE", &url, &headers, None).await
}

//  Event rute 

pub async fn event_create(State(state): State<AppState>, req: Request) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/events", state.event_url);
    forward_request(&state.client, "POST", &url, &headers, Some(body)).await
}

pub async fn event_list(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let uri = req.uri().clone();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("{}/events{}", state.event_url, query);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn event_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/events/{}", state.event_url, id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn event_update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/events/{}", state.event_url, id);
    forward_request(&state.client, "PUT", &url, &headers, Some(body)).await
}

pub async fn event_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/events/{}", state.event_url, id);
    forward_request(&state.client, "DELETE", &url, &headers, None).await
}

//  Registration rute 

pub async fn reg_create(State(state): State<AppState>, req: Request) -> Response {
    let (headers, body) = read_body(req).await;
    let url = format!("{}/registrations", state.registration_url);
    forward_request(&state.client, "POST", &url, &headers, Some(body)).await
}

pub async fn reg_my(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/registrations/my", state.registration_url);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn reg_by_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/registrations/event/{}", state.registration_url, event_id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

pub async fn reg_cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/registrations/{}", state.registration_url, id);
    forward_request(&state.client, "DELETE", &url, &headers, None).await
}

pub async fn reg_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/registrations/{}/ticket", state.registration_url, id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

/// GET registration qr
pub async fn reg_qr(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/registrations/{}/qr", state.registration_url, id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

//  Analitike 

/// GET analytics event
pub async fn analytics_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    req: Request,
) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/analytics/event/{}", state.registration_url, event_id);
    forward_request(&state.client, "GET", &url, &headers, None).await
}

/// GET analytics overview
pub async fn analytics_overview(State(state): State<AppState>, req: Request) -> Response {
    let headers = req.headers().clone();
    let url = format!("{}/analytics/overview", state.registration_url);
    forward_request(&state.client, "GET", &url, &headers, None).await
}
