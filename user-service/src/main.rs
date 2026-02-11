// User servis - upravljanje korisnickim profilima

mod db;
mod handlers;
mod models;

use axum::{routing::{get, put, delete}, Router};
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

    let database_url = std::env::var("USER_DATABASE_URL")
        .expect("USER_DATABASE_URL mora biti postavljen u .env");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET mora biti postavljen u .env");

    let pool = db::create_pool(&database_url).await;

    let state = AppState {
        db: pool,
        jwt_secret,
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/profile", get(handlers::get_my_profile))
        .route("/profile", put(handlers::upsert_profile))
        .route("/profiles", get(handlers::list_profiles))
        .route("/profiles/{id}", get(handlers::get_profile_by_id))
        .route("/profiles/{id}", delete(handlers::delete_profile))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("Ne mogu da pokrenem server na portu 3002");

    tracing::info!("User Service pokrenut na http://localhost:3002");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
