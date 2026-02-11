// Proxy - prosledjuje zahteve ka backend servisima

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;

/// Salje zahtev ka backend servisu i vraca odgovor klijentu
pub async fn forward_request(
    client: &Client,
    method: &str,
    url: &str,
    headers: &HeaderMap,
    body: Option<String>,
) -> Response {
    let mut req = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => {
            return (StatusCode::METHOD_NOT_ALLOWED, "Metod nije podrzan").into_response();
        }
    };

    // Prosledjujemo headere kao stringove (axum i reqwest koriste razlicite tipove)
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            req = req.header("authorization", val);
        }
    }

    if let Some(ct) = headers.get("content-type") {
        if let Ok(val) = ct.to_str() {
            req = req.header("content-type", val);
        }
    }

    if let Some(b) = body {
        req = req.body(b);
    }

    match req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            // preuzmi content-type od backend servisa, default na json
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let body_bytes = resp.bytes().await.unwrap_or_default();
            (status, [(axum::http::header::CONTENT_TYPE, content_type)], Body::from(body_bytes)).into_response()
        }
        Err(_) => {
            (StatusCode::SERVICE_UNAVAILABLE, "Servis nije dostupan").into_response()
        }
    }
}
