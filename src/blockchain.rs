use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::error;
use sqlx::{FromRow, SqlitePool};
use sqlx::sqlite::SqlitePoolOptions;


const DIFFICULTY: usize = 5; // Number of leading zeros required in the hash

#[derive(Clone, FromRow)]
pub struct Block {
    pub index: i32,
    pub timestamp: f64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: i32,
}


impl Block {
    pub fn new(index: i32, data: String, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
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
}


pub struct Blockchain {
    // pub blocks: Vec<Block>, // those are instead stored in the database
    // Keep only the blockchain head, becasu all other blocks (which are many and can cause memory issues)
    // are stored in the sqlite database
    pub blockchain_head: Block
}

impl Blockchain {
    pub async fn new() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.sqlite".to_string());

        let pool: SqlitePool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap_or_else(|e| {
                error!("failed to connect to SQLite at {}: {}", database_url, e);
                panic!("failed to connect to SQLite");
            });

        // Blocks will be either one or zero. Do not fetch them all as this will may cause out of memory issues
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT idx AS "index", timestamp, data, previous_hash, hash, nonce
            FROM blocks
            ORDER BY idx DESC
            LIMIT 1;
            "#,
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to insert block: {}", e);
            panic!("failed to insert block");
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

        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.sqlite".to_string());

        let pool: SqlitePool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap_or_else(|e| {
                error!("failed to connect to SQLite at {}: {}", database_url, e);
                panic!("failed to connect to SQLite");
            });

        let result = sqlx::query(
            r#"
            INSERT INTO blocks (data, previous_hash, hash, nonce)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(block.data)
        .bind(block.previous_hash)
        .bind(block.hash)
        .bind(block.nonce)
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("failed to insert block: {}", e);
            panic!("failed to insert block");
        });
    }
}
