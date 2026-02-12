// API Gateway - ulazna tacka za klijente, rutira zahteve ka servisima

mod handlers;
mod proxy;

use axum::{routing::{get, post, delete}, Router};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub auth_url: String,
    pub user_url: String,
    pub event_url: String,
    pub registration_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let state = AppState {
        client: reqwest::Client::new(),
        auth_url: std::env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string()),
        user_url: std::env::var("USER_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string()),
        event_url: std::env::var("EVENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3003".to_string()),
        registration_url: std::env::var("REGISTRATION_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3004".to_string()),
    };

    // CORS - dozvoljava frontend-u da pristupa API-ju
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health
        .route("/health", get(handlers::health_check))
        // Auth
        .route("/api/auth/register", post(handlers::auth_register))
        .route("/api/auth/login", post(handlers::auth_login))
        .route("/api/auth/me", get(handlers::auth_me))
        // Users
        .route("/api/users/profile", get(handlers::user_profile_get).put(handlers::user_profile_put))
        .route("/api/users/profiles", get(handlers::user_profiles_list))
        .route("/api/users/profiles/:id", get(handlers::user_profile_by_id).delete(handlers::user_profile_delete))
        // Events
        .route("/api/events", get(handlers::event_list).post(handlers::event_create))
        .route("/api/events/:id", get(handlers::event_get).put(handlers::event_update).delete(handlers::event_delete))
        // Registrations
        .route("/api/registrations", post(handlers::reg_create))
        .route("/api/registrations/my", get(handlers::reg_my))
        .route("/api/registrations/event/:event_id", get(handlers::reg_by_event))
        .route("/api/registrations/:id", delete(handlers::reg_cancel))
        .route("/api/registrations/:id/ticket", get(handlers::reg_ticket))
        .route("/api/registrations/:id/qr", get(handlers::reg_qr))
        // Analitike
        .route("/api/analytics/event/:event_id", get(handlers::analytics_event))
        .route("/api/analytics/overview", get(handlers::analytics_overview))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Ne mogu da pokrenem server na portu 3000");

    tracing::info!("API Gateway pokrenut na http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
