---
name: launch
description: Build and launch Harper with checks
---

# Build and Launch Harper

See AGENTS.md for full guidelines on file operations, security, and coding conventions.

## Pre-flight Checks
- Run: `cargo check`
- Run clippy: `cargo clippy -- -D warnings`
- Check fmt: `cargo fmt -- --check`
- Run tests: `cargo test`

## Build
- Release: `cargo build --release`
- Or Bazel: `bazel build //...`

## Launch
- Run binary with appropriate flags

## Core Guidelines (see AGENTS.md for details)
- Read only within project scope
- Require user consent for writes
- Never delete; use git operations
- Use Result/Option over unwrap()
- Prefer structs/enums over inheritance
- Use iterator methods (.map, .filter, .fold)