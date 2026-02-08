// Event servis - kreiranje, izmena, brisanje i pretraga dogadjaja

use axum::{routing::get, Json, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("Ne mogu da pokrenem server na portu 3003");

    tracing::info!("Event Service pokrenut na http://localhost:3003");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "service": "event-service",
        "status": "ok"
    }))
}
