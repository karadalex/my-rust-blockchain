#[macro_use] extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::tokio::{self, task};
use rocket::Shutdown;
use std::sync::{Arc, Mutex};
use sqlx::sqlite::SqlitePoolOptions;

mod blockchain;
use blockchain::{Block, Blockchain};


#[get("/")]
fn index() -> &'static str { "ok" }

async fn cpu_worker(mut shutdown: Shutdown) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mut i: u64 = 1;

    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            _ = async {
                // Run CPU work on a dedicated blocking thread pool:
                let blockchain = Arc::clone(&blockchain);
                let index = i;
                task::spawn_blocking(move || {
                    let mut blockchain = blockchain.lock().expect("blockchain lock");
                    blockchain_operations(index, &mut *blockchain);
                }).await.expect("spawn_blocking failed");
            } => {}
        }
        i += 1;
    }
}

fn blockchain_operations(i: u64, blockchain: &mut Blockchain) {
    let data = format!("Block {}", i);
    let new_block = Block::new(i, data, String::new());
    blockchain.add_block(new_block);

    if let Some(block) = blockchain.blocks.last() {
        println!(
            "Index: {}, Timestamp: {}, Data: {}, Previous Hash: {}, Hash: {}, Nonce: {}",
            block.index, block.timestamp, block.data, block.previous_hash, block.hash, block.nonce
        );
    }
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    
    // For local dev this could be e.g. "sqlite://app.db"
    // Or "sqlite::memory:" for in-memory testing.
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.sqlite".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            error!("failed to connect to SQLite at {}: {}", database_url, e);
            panic!("failed to connect to SQLite");
        });

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    rocket::build()
        .mount("/", routes![index])
        .attach(AdHoc::on_liftoff("spawn cpu worker", |rocket| {
            Box::pin(async move {
                tokio::spawn(cpu_worker(rocket.shutdown()));
            })
        }))
}
