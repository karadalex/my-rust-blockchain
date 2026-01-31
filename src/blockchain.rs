use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};


const DIFFICULTY: usize = 4; // Number of leading zeros required in the hash

pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}


impl Block {
    pub fn new(index: u64, data: String, previous_hash: String) -> Self {
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
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain { blocks: vec![] };
        let genesis_block = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        blockchain.add_block(genesis_block);
        blockchain
    }

    pub fn add_block(&mut self, mut block: Block) {
        if let Some(last_block) = self.blocks.last() {
            block.previous_hash = last_block.hash.clone();
        }
        block.mine();
        self.blocks.push(block);
    }
}