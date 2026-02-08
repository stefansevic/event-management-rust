// User servis - CRUD operacije nad korisnickim profilima

use axum::{routing::get, Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("Ne mogu da pokrenem server na portu 3002");

    tracing::info!("User Service pokrenut na http://localhost:3002");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "user-service",
        "status": "ok"
    }))
}
