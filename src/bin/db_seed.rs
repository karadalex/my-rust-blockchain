use rocket_db_pools::sqlx::{self, sqlite::SqlitePoolOptions};
use my_rust_blockchain::utils::*;

#[rocket::tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = db_pool().await;

  
    Ok(())
}