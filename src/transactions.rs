use sqlx::FromRow;
use crate::utils::*;
use rocket::{error, get, post, serde::json::Json, routes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};


pub fn routes() -> Vec<rocket::Route> {
    routes![create_transaction]
}


#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: i32,
    pub sig: Option<String>,
    pub added_to_block: Option<bool>,
    pub created_at: Option<f64>
}

#[derive(Clone, FromRow)]
pub struct Wallet {
    pub address: String,
    pub balance: i32,
    pub pub_key: String
}

impl Transaction {
    pub fn new() -> Self {
        todo!()
    }
}

impl Wallet {
    pub fn new() -> Self {
        todo!()
    }
}


#[post("/tx", data="<transaction>")]
async fn create_transaction(transaction: Json<Transaction>) ->ApiResult<Transaction> {
    let pool = db_pool().await;

    let mut hasher = Sha256::new();
    hasher.update(&transaction.from_address);
    hasher.update(&transaction.to_address);
    hasher.update(transaction.amount.to_string());
    let signature = format!("{:x}", hasher.finalize());

    // TODO: Check if addresses exist

    let transaction: Transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (from_address, to_address, amount, sig, added_to_block)
        VALUES (?, ?, ?, ?, ?)
        RETURNING from_address, to_address, amount, sig, added_to_block, created_at;
        "#
    )
    .bind(&transaction.from_address)
    .bind(&transaction.to_address)
    .bind(transaction.amount)
    .bind(signature)
    .bind(false)
    .fetch_one(&pool)
    .await
    .unwrap_or_else(|e| {
        error!("failed to get transaction: {}", e);
        panic!("failed to get transaction");
    });

    Ok(Json(transaction))
}