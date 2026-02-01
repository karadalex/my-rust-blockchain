My-Rust-Blockchain
==================

## Instructions

```bash
cargo update
touch database.sqlite
sqlx migrate run --database-url sqlite://database.sqlite
cargo run
cargo run --bin db_seed
```

## TODO

1. [ ] GET /health
2. [ ] GET /chain/head
3. [x] GET /chain/height
4. [ ] GET /block/{hash}
5. [x] POST /tx (optional)
6. [ ] POST /mine or /produce_block (if node can author blocks) 
7. [ ] On new block creation check if there are pending transactions from a transactions table
8. [ ] On boot check if the database state is correct and not corrupted
9. [ ] Support account balances
10. [x] Store blockchain on database
11. [ ] Do not allow changing difficulty and get value and increment based on state
12. [ ] Merkle root of a block transactions
13. [ ] Database seeding with test data
14. [ ] Allow multiple miners and reward the first one who finds the nonce
15. [ ] GET /wallet/{address}

## References

- [https://www.youtube.com/watch?v=1oJrLNKSVf8](https://www.youtube.com/watch?v=1oJrLNKSVf8)