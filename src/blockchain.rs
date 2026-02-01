use rand::seq::index;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use sqlx::FromRow;
use crate::utils::*;
use rocket::{error, get, serde::json::Json, routes};
use serde::{Deserialize, Serialize};
use crate::transactions::Transaction;


const DIFFICULTY: usize = 5; // Number of leading zeros required in the hash

pub fn routes() -> Vec<rocket::Route> {
    routes![get_chain_height, get_block_by_hash, get_block_transactions]
}

#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct Block {
    pub idx: i32,
    pub timestamp: f64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: i32,
}


impl Block {
    pub fn new(idx: i32, data: String, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Block {
            idx,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.idx.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.data);
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self) {
        while !self.hash.starts_with(&"0".repeat(DIFFICULTY)) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }

    pub async fn prepare_unmined_block(&mut self) {
        let pool = db_pool().await;

        // Blocks will be either one or zero. Do not fetch them all as this will may cause out of memory issues
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT *
            FROM transactions
            WHERE block_id IS NULL OR block_id = '';
            "#,
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to get block: {}", e);
            panic!("failed to get block");
        });

        let json_data = serde_json::to_string(&transactions).unwrap();
        let merkle_root = self.merkle_root(transactions);
        self.data = format!("{}{}", merkle_root, json_data);
    }

    pub async fn get_transactions(&mut self) -> Vec<Transaction> {
        let pool = db_pool().await;

        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT *
            FROM transactions
            WHERE block_id = ?;
            "#,
        )
        .bind(self.idx)
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to get block: {}", e);
            panic!("failed to get block");
        });

        transactions
    }

    pub fn merkle_root(&mut self, transactions: Vec<Transaction>) -> String {
        if transactions.is_empty() {
            let mut hasher = Sha256::new();
            hasher.update("");
            return format!("{:x}", hasher.finalize());
        }

        let mut hashes: Vec<String> = transactions
            .into_iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                let json = serde_json::to_string(&tx).unwrap_or_default();
                hasher.update(json);
                format!("{:x}", hasher.finalize())
            })
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                let last = hashes.last().cloned().unwrap();
                hashes.push(last);
            }

            let mut next = Vec::with_capacity(hashes.len() / 2);
            for pair in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);
                hasher.update(&pair[1]);
                next.push(format!("{:x}", hasher.finalize()));
            }
            hashes = next;
        }

        hashes.pop().unwrap()
    }
}


pub struct Blockchain {
    // pub blocks: Vec<Block>, // those are instead stored in the database
    // Keep only the blockchain head, becasu all other blocks (which are many and can cause memory issues)
    // are stored in the sqlite database
    pub blockchain_head: Block
}

impl Blockchain {
    pub async fn new() -> Self {
        let pool = db_pool().await;

        // Blocks will be either one or zero. Do not fetch them all as this will may cause out of memory issues
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT idx, timestamp, data, previous_hash, hash, nonce
            FROM blocks
            ORDER BY idx DESC
            LIMIT 1;
            "#,
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to get block: {}", e);
            panic!("failed to get block");
        });

        // initiliaze blockchain but undo if there are records in the datbase
        let genesis_block = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        let mut blockchain = Blockchain { blockchain_head: genesis_block.clone() };
        if blocks.is_empty() {
            blockchain.add_block(genesis_block.clone()).await;
        } else if let Some(last_block) = blocks.last() {
            blockchain.blockchain_head = last_block.clone();
        }
        blockchain
    }

    pub async fn add_block(&mut self, mut block: Block) {
        block.mine();
        block.previous_hash = self.blockchain_head.clone().hash;

        let pool = db_pool().await;

        let result = sqlx::query(
            r#"
            INSERT INTO blocks (data, previous_hash, hash, nonce)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&block.data)
        .bind(&block.previous_hash)
        .bind(&block.hash)
        .bind(block.nonce)
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to insert block: {}", e);
            panic!("failed to insert block");
        });

        self.blockchain_head = block.clone();
    }

    pub async fn get_height(&mut self) -> i32 {
        let pool = db_pool().await;

        sqlx::query_scalar::<_, i32>(
        r#"
            SELECT COUNT(*) FROM blocks;        
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to get chain size: {}", e);
            panic!("failed to get chain size");
        })

    }
}


#[get("/chain/height")]
async fn get_chain_height() -> ApiResult<DataBody<i32>> {
    let blockchain = Blockchain::new();
    let height = DataBody { data: blockchain.await.get_height().await };

    Ok(Json(height))
}


#[get("/chain/<id>")]
async fn get_block_by_hash(id: i32) ->ApiResult<Block> {
    let pool = db_pool().await;

    let block: Block = sqlx::query_as::<_, Block>(
        r#"
        SELECT * FROM blocks
        WHERE idx = ?;
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or_else(|e| {
        error!("failed to get block: {}", e);
        panic!("failed to get block");
    });

    Ok(Json(block))
}


#[get("/chain/<id>/txs")]
async fn get_block_transactions(id: i32) -> ApiResult<Vec<Transaction>> {
    let pool = db_pool().await;

    let mut block: Block = sqlx::query_as::<_, Block>(
        r#"
        SELECT * FROM blocks
        WHERE idx = ?;
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or_else(|e| {
        error!("failed to get block: {}", e);
        panic!("failed to get block");
    });

    let transactions = block.get_transactions().await;

    Ok(Json(transactions))
}