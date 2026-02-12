// Registration servis - prijave na dogadjaje, karte, QR kodovi, analitike

mod db;
mod handlers;
mod models;

use axum::{routing::{get, post, delete}, Router};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub event_service_url: String,
    pub qr_service_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("REGISTRATION_DATABASE_URL")
        .expect("REGISTRATION_DATABASE_URL mora biti postavljen u .env");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET mora biti postavljen u .env");

    let event_service_url = std::env::var("EVENT_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3003".to_string());

    let qr_service_url = std::env::var("QR_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3005".to_string());

    let pool = db::create_pool(&database_url).await;

    let state = AppState {
        db: pool,
        jwt_secret,
        event_service_url,
        qr_service_url,
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/registrations", post(handlers::register_for_event))
        .route("/registrations/my", get(handlers::my_registrations))
        .route("/registrations/event/:event_id", get(handlers::event_registrations))
        .route("/registrations/:id", delete(handlers::cancel_registration))
        .route("/registrations/:id/ticket", get(handlers::get_ticket))
        .route("/registrations/:id/qr", get(handlers::get_ticket_qr))
        .route("/analytics/event/:event_id", get(handlers::analytics_event))
        .route("/analytics/overview", get(handlers::analytics_overview))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004")
        .await
        .expect("Ne mogu da pokrenem server na portu 3004");

    tracing::info!("Registration Service pokrenut na http://localhost:3004");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
