use rocket::error;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub async fn db_pool() -> SqlitePool {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.sqlite".to_string());

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            error!("failed to connect to SQLite at {}: {}", database_url, e);
            panic!("failed to connect to SQLite");
        })
}
