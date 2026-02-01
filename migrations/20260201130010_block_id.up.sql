-- Add up migration script here
ALTER TABLE transactions
ADD COLUMN block_id INTEGER REFERENCES blocks(idx);
