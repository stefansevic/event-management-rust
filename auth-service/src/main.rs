// Auth servis 

mod db;
mod handlers;
mod models;

use axum::{routing::{get, post}, Router};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

///Seeduj admina ako ne postoji 
async fn seed_admin(pool: &PgPool) {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = 'saske@admin.com')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !exists {
        let password_hash = bcrypt::hash("saske1", bcrypt::DEFAULT_COST)
            .expect("Greska pri hesiranju lozinke");

        let _ = sqlx::query(
            "INSERT INTO users (id, email, password_hash, role) VALUES (gen_random_uuid(), 'saske@admin.com', $1, 'Admin')"
        )
        .bind(&password_hash)
        .execute(pool)
        .await;

        tracing::info!("Admin nalog kreiran (saske@admin.com / saske1)");
    } else {
        tracing::info!("Admin nalog vec postoji, preskacemo seed");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("AUTH_DATABASE_URL")
        .expect("AUTH_DATABASE_URL mora biti postavljen u .env");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET mora biti postavljen u .env");

    let pool = db::create_pool(&database_url).await;

    // seeduj admina
    seed_admin(&pool).await;

    let state = AppState {
        db: pool,
        jwt_secret,
    };

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/me", get(handlers::me))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Ne mogu da pokrenem server na portu 3001");

    tracing::info!("Auth Service pokrenut na http://localhost:3001");

    axum::serve(listener, app)
        .await
        .expect("Greska pri pokretanju servera");
}
