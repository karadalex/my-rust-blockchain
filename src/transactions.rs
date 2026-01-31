use sqlx::FromRow;
use crate::utils::*;


#[derive(Clone, FromRow)]
pub struct Transaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub sig: String,
    pub created_at: f64
}

impl Transaction {
    pub fn new() -> Self {
        todo!()
    }
}