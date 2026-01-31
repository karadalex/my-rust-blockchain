-- Add up migration script here
CREATE TABLE IF NOT EXISTS blocks (
  idx INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp BIGINT NOT NULL DEFAULT (strftime('%s','now') * 1000), -- Block.timestamp (u128 in Rust; store as BIGINT)
  data TEXT   NOT NULL,
  previous_hash TEXT   NOT NULL,
  hash TEXT   NOT NULL,
  nonce BIGINT NOT NULL
);