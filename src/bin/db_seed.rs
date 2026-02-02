use rocket_db_pools::sqlx;
use my_rust_blockchain::utils::*;
use rand::{distr::Alphanumeric, Rng};
use rocket::error;


#[rocket::tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = db_pool().await;

    for _i in 1..20 {
        let address: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(20) // Length of the string
            .map(char::from)
            .collect();
        let pub_key: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(20) // Length of the string
            .map(char::from)
            .collect();
        let mut rng = rand::rng();

        let _result = sqlx::query(
            r#"
            INSERT INTO wallets (address, balance, pub_key)
            VALUES (?, ?, ?);
            "#,
        )
        .bind(address)
        .bind(rng.random_range(1..10000))
        .bind(pub_key)
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to insert wallet: {}", e);
            panic!("failed to insert wallet");
        });
    }
  
    Ok(())
}
