-- Add down migration script here
ALTER TABLE transactions
DROP COLUMN block_id;