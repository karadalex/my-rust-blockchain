use sqlx::FromRow;
use crate::utils::*;


#[derive(Clone, FromRow)]
pub struct Transaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: i32,
    pub sig: String,
    pub created_at: f64
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