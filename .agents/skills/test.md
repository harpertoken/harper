---
name: test
description: Run tests for Harper
---

# Run Tests

## Unit Tests
- Run: `cargo test`

## With Output
- `cargo test -- --nocapture`

## Specific Test
- `cargo test <test_name>`

## Integration Tests
- `cargo test --test integration`
- `cargo test --test integration_cli`

## All Tests (including benchmarks)
- `cargo test --all`

## With Coverage
- `cargo tarpaulin --out Html`
