

mod db;
mod handlers;
mod models;

use axum::{routing::get, Router};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("EVENT_DATABASE_URL")
        .expect("EVENT_DATABASE_URL mora biti postavljen u .env");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET mora biti postavljen u .env");

    let pool = db::create_pool(&database_url).await;

    let state = AppState {
        db: pool,
        jwt_secret,
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/events", get(handlers::list_events).post(handlers::create_event))
        .route("/events/:id", get(handlers::get_event).put(handlers::update_event).delete(handlers::delete_event))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("Ne mogu da pokrenem server na portu 3003");

    tracing::info!("Event Service pokrenut na http://localhost:3003");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
