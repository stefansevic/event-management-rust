// Auth servis - registracija, login, JWT tokeni, uloge

use axum::{routing::get, Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Ne mogu da pokrenem server na portu 3001");

    tracing::info!("Auth Service pokrenut na http://localhost:3001");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}

/// Provera da li servis radi
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "auth-service",
        "status": "ok"
    }))
}
