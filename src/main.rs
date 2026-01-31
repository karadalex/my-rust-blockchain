use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;


const DIFFICULTY: usize = 4; // Number of leading zeros required in the hash

struct Block {
    index: u64,
    timestamp: u128,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}


impl Block {
    fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.data);
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    fn mine(&mut self) {
        while !self.hash.starts_with(&"0".repeat(DIFFICULTY)) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }
}


struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let mut blockchain = Blockchain { blocks: vec![] };
        let genesis_block = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        blockchain.add_block(genesis_block);
        blockchain
    }

    fn add_block(&mut self, mut block: Block) {
        if let Some(last_block) = self.blocks.last() {
            block.previous_hash = last_block.hash.clone();
        }
        block.mine();
        self.blocks.push(block);
    }
}

fn main() {
    let mut blockchain = Blockchain::new();

    let mut i = 1;
    loop {
        let data = format!("Block {}", i);
        let new_block = Block::new(i, data, String::new());
        blockchain.add_block(new_block);
        if let Some(block) = blockchain.blocks.last() {
            println!(
                "Index: {}, Timestamp: {}, Data: {}, Previous Hash: {}, Hash: {}, Nonce: {}",
                block.index, block.timestamp, block.data, block.previous_hash, block.hash, block.nonce
            );
        }
        i += 1;
    }
}