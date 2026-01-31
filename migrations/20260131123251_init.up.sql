-- Add up migration script here
CREATE TABLE IF NOT EXISTS blocks (
  idx INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp REAL NOT NULL DEFAULT (strftime('%f','now') * 1000.0),
  data TEXT   NOT NULL,
  previous_hash TEXT   NOT NULL,
  hash TEXT   NOT NULL,
  nonce BIGINT NOT NULL
);