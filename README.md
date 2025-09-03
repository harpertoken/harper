# Harper

[![CI Status](https://github.com/harpertoken/harper/actions/workflows/ci.yml/badge.svg)](https://github.com/harpertoken/harper/actions)
[![Security Audit](https://github.com/harpertoken/harper/actions/workflows/security.yml/badge.svg)](https://github.com/harpertoken/harper/actions)
[![Release](https://img.shields.io/github/v/release/harpertoken/harper)](https://github.com/harpertoken/harper/releases)

Rust-based AI agent for multi-provider integration, command execution, and MCP protocol support with local SQLite storage.

## Latest Release: v1.3.0

- Resolved CodeQL dependency conflicts (20+ → 9 minor conflicts)
- Fixed CI build failures across all platforms (Linux, Windows, macOS)
- Enhanced security analysis with improved DevSkim integration
- Cross-platform compatibility verified on all supported architectures

## Requirements

- Rust 1.70.0+
- Network connectivity for API calls
- Linux, macOS, or Windows
- SQLite3 for data persistence

## Installation

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/harpertoken/harper.git
cd harper
cargo build --release

# Configure environment
cp env.example .env
# Set API keys in .env file

# Run
cargo run --release
```

### Install from Release

```bash
cargo install --git https://github.com/harpertoken/harper.git --tag v1.3
```

## Usage

```text
[SEARCH: query]
[RUN_COMMAND command]
[TOOL: name] { "param": "value" }
```

## Features

### AI Provider Integration

| Provider  | Model                      | Capabilities              |
|-----------|----------------------------|---------------------------|
| OpenAI    | GPT-4 Turbo               | Text generation, coding   |
| Sambanova | Meta-Llama-3.2-1B-Instruct| Open-source LLM           |
| Gemini    | Gemini 2.0 Flash          | Multimodal processing     |

### Capabilities

- Command execution with validation
- Web search integration
- Session management with SQLite
- Session export functionality
- Interactive CLI interface

### Security

- CodeQL vulnerability detection
- DevSkim security scanning
- Dependency auditing
- AES-GCM-256 encryption
- Input validation

### Model Context Protocol

MCP functionality is temporarily disabled in v1.3.0 to resolve dependency conflicts.

```toml
[mcp]
enabled = true
server_url = "http://localhost:5000"
```

### Data Storage

- SQLite database for conversations
- Local environment for credentials
- Session persistence
- Export functionality

## Build Commands

| Command                              | Description               |
|--------------------------------------|---------------------------|
| `cargo build --release`             | Release build             |
| `cargo run --release`               | Run release binary        |
| `cargo test`                        | Run tests                 |
| `cargo fmt -- --check`              | Check formatting          |
| `cargo clippy`                      | Static analysis           |
| `cargo doc`                         | Generate documentation    |
| `cargo clean`                       | Clean build artifacts     |

### Cross-Platform Builds

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS ARM
cargo build --release --target aarch64-apple-darwin
```

## Configuration

Create `.env` file:

```bash
OPENAI_API_KEY=key
SAMBASTUDIO_API_KEY=key
GEMINI_API_KEY=key
DATABASE_PATH=./harper.db
```

Edit `config/default.toml`:

```toml
[api]
timeout = 90
retry_attempts = 3

[cache]
enabled = true
ttl_seconds = 300
```

## Security

- Local SQLite storage only
- No data transmission to external servers
- Environment-based credential storage
- AES-GCM-256 encryption
- CodeQL vulnerability detection
- DevSkim security scanning
- Dependency auditing
- Input validation
- Memory safety guarantees

## Recent Updates (v1.3.0)

- Fixed 20+ CodeQL dependency conflicts
- Resolved CI build failures across all platforms
- Enhanced security analysis with DevSkim integration
- Verified cross-platform compatibility

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cargo fetch
cargo test
cargo clippy
cargo fmt -- --check
```

### Issues

- Bug Reports: [GitHub Issues](https://github.com/harpertoken/harper/issues)
- Security Issues: [Security Policy](SECURITY.md)

## License

Apache 2.0 - See [LICENSE](LICENSE)

## Links

- [Contributing Guide](CONTRIBUTING.md)
- [Privacy Policy](PRIVACY.md)
- [Security Policy](SECURITY.md)