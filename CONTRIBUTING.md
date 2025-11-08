# Contributing

## Setup

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cargo build
cp env.example .env
cargo test

# Optional: Set up git hooks for commit validation
cp scripts/commit-msg .git/hooks/commit-msg && chmod +x .git/hooks/commit-msg
cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

---

## Development

```bash
# Run checks
cargo test --all-features --workspace
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo fmt --all -- --check

# Format and commit
cargo fmt --all
git add .
git commit -m "Brief description"
git push
```

---

## Code Style

- Follow Rust API Guidelines
- Use `cargo fmt` for formatting
- Resolve all `cargo clippy` warnings
- Document public APIs
- Write tests for new functionality

---

## Issues

Include:
- Version: `cargo run -- --version`
- OS and steps to reproduce
- Expected vs actual behavior
- Error logs

## Adding Providers

1. Add to `ApiProvider` enum
2. Update `ApiConfig` struct
3. Implement in `call_llm()`
4. Update documentation

## License

Apache 2.0 - [LICENSE](LICENSE)