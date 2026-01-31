-- Add up migration script here
CREATE TABLE IF NOT EXISTS transactions (
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount INTEGER NOT NULL,
    sig TEXT NOT NULL,
    created_at REAL NOT NULL DEFAULT (strftime('%f','now') * 1000.0)
);
