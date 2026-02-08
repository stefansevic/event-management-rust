// Auth servis - registracija, login, JWT tokeni

mod db;
mod handlers;
mod models;

use axum::{routing::{get, post}, Router};
use sqlx::PgPool;

/// Stanje aplikacije - deli se izmedju svih handler-a
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Ucitavamo .env fajl (ako postoji)
    dotenvy::dotenv().ok();

    let database_url = std::env::var("AUTH_DATABASE_URL")
        .expect("AUTH_DATABASE_URL mora biti postavljen u .env");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET mora biti postavljen u .env");

    // Povezujemo se na bazu
    let pool = db::create_pool(&database_url).await;

    let state = AppState {
        db: pool,
        jwt_secret,
    };

    // Definisemo rute
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Ne mogu da pokrenem server na portu 3001");

    tracing::info!("Auth Service pokrenut na http://localhost:3001");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
