use rocket::error;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DataBody<T> {
    pub data: T,
}

#[macro_export]
macro_rules! error_response {
    ($status:expr, $msg:expr) => {
        (
            $status,
            Json($crate::utils::ErrorBody {
                message: $msg.to_string(),
            }),
        )
    };
}

pub type ApiResult<T> = Result<Json<T>, (Status, Json<ErrorBody>)>;

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
