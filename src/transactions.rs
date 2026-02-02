use sqlx::FromRow;
use crate::utils::*;
use crate::error_response;
use rocket::{error, get, post, serde::json::Json, routes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use rocket::http::Status;


pub fn routes() -> Vec<rocket::Route> {
    routes![create_transaction, get_wallet_details]
}


#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: i32,
    pub sig: Option<String>,
    pub added_to_block: Option<bool>,
    pub created_at: Option<f64>,
    pub block_id: Option<i32>
}

#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: i32,
    pub pub_key: String
}

impl Transaction {
    // pub fn new() -> Self {
    //     todo!()
    // }

    pub async fn is_valid(&self) -> Result<bool, (Status, Json<ErrorBody>)> {
        let pool = db_pool().await;

        let from_wallet = sqlx::query_as::<_, Wallet>(
        r#"
                SELECT *
                FROM wallets
                WHERE address = ?;
                "#,
            )
            .bind(&self.from_address)
            .fetch_optional(&pool)
            .await
            .unwrap_or_else(|e| {
                error!("failed to get block: {}", e);
                panic!("failed to get block");
            });
        let from_wallet = match from_wallet {
            Some(wallet) => wallet,
            None => return Ok(false),
        };
        if self.amount > from_wallet.balance {
            return Ok(false);
        }

        let to_exists: i64 = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM wallets
                WHERE address = ?
            );
            "#,
        )
        .bind(&self.to_address)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("failed to check to wallet: {}", e);
            error_response!(Status::InternalServerError, "failed to check to wallet")
        })?;
        if to_exists == 0 {
            return Ok(false);
        }

        if self.amount < 0 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// impl Wallet {
//     pub fn new() -> Self {
//         todo!()
//     }
// }


#[post("/tx", data="<transaction>")]
async fn create_transaction(transaction: Json<Transaction>) ->ApiResult<Transaction> {
    let pool = db_pool().await;
    let is_valid_tx = transaction.is_valid().await?;
    if !is_valid_tx {
        return Err(error_response!(Status::NotFound, "transaction not valid"))
    }

    // Calculate transaction hash (will bre raplaced with signateure calculated with private key)
    let mut hasher = Sha256::new();
    hasher.update(&transaction.from_address);
    hasher.update(&transaction.to_address);
    hasher.update(transaction.amount.to_string());
    let signature = format!("{:x}", hasher.finalize());

    // Reduce from_wallet and increase to_wallet by the same amount
    let from_result = sqlx::query(
        r#"
        UPDATE wallets
        SET balance = balance - ?
        WHERE address = ?
        RETURNING *;
        "#,
    )
    .bind(&transaction.amount)
    .bind(&transaction.from_address)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("failed to update from wallet: {}", e);
        error_response!(Status::InternalServerError, "failed to update from wallet")
    })?;
    if from_result.rows_affected() == 0 {
        return Err(error_response!(Status::NotFound, "from wallet not found"));
    }

    let to_result = sqlx::query(
        r#"
        UPDATE wallets
        SET balance = balance + ?
        WHERE address = ?
        RETURNING *;
        "#,
    )
    .bind(&transaction.amount)
    .bind(&transaction.to_address)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("failed to update to wallet: {}", e);
        error_response!(Status::InternalServerError, "failed to update to wallet")
    })?;
    if to_result.rows_affected() == 0 {
        return Err(error_response!(Status::NotFound, "to wallet not found"));
    }

    
    // Create transaction
    let transaction: Transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (from_address, to_address, amount, sig, added_to_block)
        VALUES (?, ?, ?, ?, ?)
        RETURNING from_address, to_address, amount, sig, added_to_block, created_at, block_id;
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


#[get("/wallet/<address>")]
async fn get_wallet_details(address: String) -> ApiResult<Wallet> {
    let pool = db_pool().await;

    let wallet = sqlx::query_as::<_, Wallet>(
        r#"
        SELECT *
        FROM wallets
        WHERE address = ?;
        "#,
    )
    .bind(address)
    .fetch_one(&pool)
    .await
    .unwrap_or_else(|e| {
        error!("failed to get block: {}", e);
        panic!("failed to get block");
    });

    Ok(Json(wallet))
}
