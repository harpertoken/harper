<p align="center">
  <img src="https://raw.githubusercontent.com/Coccinella-Labs/harper/main/.github/assets/thumbnail.png" alt="harper" width="100%">
</p>

# harper

Fast text tools in Rust. Simple to use.

## Scope

Rust workspace. Version 0.21.0.

- `lib/harper-core` - core text logic
- `lib/harper-ui` - bins `harper` and `harper-batch`
- `lib/harper-mcp-server` - MCP server
- `lib/harper-firmware` - firmware support
- `lib/harper-sandbox` - sandbox support

## Start

```bash
cargo build --release -p harper-ui --bin harper --bin harper-batch
./target/release/harper --help
```

Or install on Mac:

```bash
brew tap coccinella-labs/homebrew-tap
brew install harper-ai
```

## Test

```bash
cargo test --workspace
```

## Contribute

- Report: open an issue for bugs or requests.
- Change: small pull requests preferred.
