-- Add up migration script here
CREATE TABLE IF NOT EXISTS wallets (
    address TEXT NOT NULL,
    balance INTEGER NOT NULL DEFAULT 0,
    pub_key TEST NOT NULL
);