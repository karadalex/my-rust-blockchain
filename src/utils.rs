use rocket::error;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;
use futures::TryStreamExt;
use sqlx::Row;


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


/// Memory-efficient DB verification: PRAGMA integrity_check + streaming block linkage check.
/// Does not load all blocks into memory.
pub async fn verify_db_state_streaming() -> Result<(), String> {
    let pool = db_pool().await;

    // 1) SQLite integrity check (note: this may still be slow on very large DBs)
    let integrity_row = sqlx::query("PRAGMA integrity_check;")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("PRAGMA integrity_check query failed: {}", e))?;

    let integrity_result: String = integrity_row
        .try_get(0)
        .map_err(|e| format!("failed to read integrity_check result: {}", e))?;

    if integrity_result.to_lowercase() != "ok" {
        return Err(format!("sqlite integrity_check failed: {}", integrity_result));
    }

    // 2) Streaming application-level chain linkage check
    // Iterate rows ordered by idx, one row at a time (no fetch_all).
    let mut stream = sqlx::query!(
        r#"
        SELECT idx, hash, previous_hash
        FROM blocks
        ORDER BY idx ASC
        "#
    )
    .fetch(&pool);

    let mut expected_idx: i64 = 1;
    let mut last_hash = String::new();

    while let Some(row) = stream
        .try_next()
        .await
        .map_err(|e| format!("failed reading blocks during verification: {}", e))?
    {
        let idx = row.idx;
        let hash = row.hash;
        let previous_hash = row.previous_hash;

        if idx != expected_idx {
            return Err(format!(
                "block index mismatch at row with idx {}: expected {}",
                idx, expected_idx
            ));
        }

        if expected_idx == 1 {
            // genesis block expected previous_hash == "0"
            if previous_hash != "" {
                return Err(format!(
                    "genesis block previous_hash invalid: got '{}', expected '0'",
                    previous_hash
                ));
            }
        } else if previous_hash != last_hash {
            return Err(format!(
                "previous_hash mismatch at idx {}: expected '{}', got '{}'",
                idx, last_hash, previous_hash
            ));
        }

        last_hash = hash;
        expected_idx += 1;
    }

    Ok(())
}