// Registration servis - prijave na dogadjaje, karte, kapacitet

use axum::{routing::get, Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004")
        .await
        .expect("Ne mogu da pokrenem server na portu 3004");

    tracing::info!("Registration Service pokrenut na http://localhost:3004");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "registration-service",
        "status": "ok"
    }))
}
