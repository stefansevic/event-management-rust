// klijent -> gateway -> servis

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;

/// Posalji req servisu i vrati odgovor
pub async fn forward_request(
    client: &Client,
    method: &str,
    url: &str,
    headers: &HeaderMap,
    body: Option<String>,
) -> Response {
    // Pravimo req
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

    // Prosledjujemo header 
    if let Some(auth) = headers.get("authorization") {
        req = req.header("authorization", auth);
    }

    // Prosledjujemo Content-Type
    if let Some(ct) = headers.get("content-type") {
        req = req.header("content-type", ct);
    }

    // Dodajemo body 
    if let Some(b) = body {
        req = req.body(b);
    }

    // Saljemo req i vracamo odgovor
    match req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let body_bytes = resp.bytes().await.unwrap_or_default();
            (status, [(axum::http::header::CONTENT_TYPE, "application/json")], Body::from(body_bytes)).into_response()
        }
        Err(_) => {
            (StatusCode::SERVICE_UNAVAILABLE, "Servis nije dostupan").into_response()
        }
    }
}
