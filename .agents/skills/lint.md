---
name: lint
description: Lint and format Harper codebase
---

# Lint and Format

## Format Check
- `cargo fmt -- --check`

## Format Fix
- `cargo fmt`

## Clippy Lints
- `cargo clippy -- -D warnings`

## All Checks
- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo fmt -- --check`

## Fix Warnings
- `cargo clippy --fix -- -D warnings`