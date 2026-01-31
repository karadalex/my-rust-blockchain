#[macro_use] extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::tokio::{self, task};
use rocket::Shutdown;
use std::sync::{Arc, Mutex};
use sqlx::sqlite::SqlitePoolOptions;

mod blockchain;
mod utils;
mod transactions;
use blockchain::{Block, Blockchain};


#[get("/")]
fn index() -> &'static str { "ok" }

async fn cpu_worker(mut shutdown: Shutdown) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new().await));

    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            _ = async {
                // Run CPU work on a dedicated blocking thread pool:
                let blockchain = Arc::clone(&blockchain);
                task::spawn_blocking(move || {
                    let mut blockchain = blockchain.lock().expect("blockchain lock");
                    rocket::tokio::runtime::Handle::current().block_on(async {
                        blockchain_operations(&mut *blockchain).await;
                    });
                }).await.expect("spawn_blocking failed");
            } => {}
        }
    }
}

async fn blockchain_operations(blockchain: &mut Blockchain) {
    let data = "Block {}".to_string();
    let index = blockchain.get_height().await;
    let new_block = Block::new(index, data, String::new());
    blockchain.add_block(new_block.clone()).await;

    println!(
        "Index: {}, Timestamp: {}, Data: {}, Previous Hash: {}, Hash: {}, Nonce: {}",
        new_block.index, new_block.timestamp, new_block.data, new_block.previous_hash, new_block.hash, new_block.nonce
    );
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
        .mount("/", blockchain::routes())
        .attach(AdHoc::on_liftoff("spawn cpu worker", |rocket| {
            Box::pin(async move {
                tokio::spawn(cpu_worker(rocket.shutdown()));
            })
        }))
}
