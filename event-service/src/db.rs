// Connection na PostgreSQL 

use sqlx::postgres::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url)
        .await
        .expect("Ne mogu da se povezem na bazu podataka")
}
