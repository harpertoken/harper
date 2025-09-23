# Harper

[![End-to-End Tests](https://github.com/harpertoken/harper/actions/workflows/e2e.yml/badge.svg)](https://github.com/harpertoken/harper/actions/workflows/e2e.yml)
[![Release](https://img.shields.io/github/v/release/harpertoken/harper)](https://github.com/harpertoken/harper/releases)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.82.0+-blue)](https://rust-lang.org)

AI agent for multi-provider integration, command execution, and MCP protocol support with SQLite storage.

## Requirements

- Rust 1.82.0+
- Network connectivity
- Linux, macOS, or Windows
- SQLite3

## Installation

<details>
<summary>Local Build</summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/harpertoken/harper.git
cd harper
cargo build --release
cp env.example .env
cargo run --release
```
</details>

<details>
<summary>Docker</summary>

Harper supports Docker for easy deployment.

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp env.example .env
# Edit .env with your API keys
docker build -t harper .
docker run --rm -it --env-file .env -v harper_data:/app/data harper
```

For detailed instructions, see [DOCKER.md](DOCKER.md).

Docker builds are validated in CI via GitHub Actions.
</details>

## Development

### Running Tests

harper includes a comprehensive test suite. To run all tests, use the provided script:

```bash
./harpertest
```

This will run:
- Unit tests
- Integration tests
- Error handling tests
- Security tests
- Performance benchmarks

Example output:
```
Running all tests and benchmarks...
==================================

Running unit tests...
...

Running integration tests...
...

Running error handling tests...
...

Running security tests...
...

Running performance benchmarks...
...

All tests completed successfully!
==================================
```

### Test Coverage

The test suite includes:
- 12 unit tests
- 10 integration tests
- 6 error handling tests
- 3 security tests
- Performance benchmarks

<details>
<summary>Install from Release</summary>

```bash
cargo install --git https://github.com/harpertoken/harper.git --tag v0.1.5
```
</details>

## Usage

```text
[SEARCH: query]
[RUN_COMMAND command]
[TOOL: name] { "param": "value" }
```

## Features

### AI Providers

| Provider  | Model                      | Capabilities              |
|-----------|----------------------------|---------------------------|
| OpenAI    | GPT-4 Turbo               | Text generation, coding   |
| Sambanova | Meta-Llama-3.2-1B-Instruct| Open-source LLM           |
| Gemini    | Gemini 2.0 Flash          | Multimodal processing     |

### Core Functions

- Command execution
- Web search
- SQLite sessions
- Session export
- CLI

### Security

- CodeQL scanning
- DevSkim analysis
- Dependency audit
- AES-GCM encryption
- Input validation

### MCP Protocol

MCP disabled in v0.1.3+ due to dependency conflicts.

<details>
<summary>MCP Configuration</summary>

```toml
[mcp]
enabled = true
server_url = "http://localhost:5000"
```
</details>

### Data Storage

- SQLite database
- Local credentials
- Session persistence
- Export functionality

## Build

| Command                | Function              |
|------------------------|-----------------------|
| `cargo build --release` | Release build        |
| `cargo run --release`  | Run binary           |
| `cargo test`           | Run tests            |
| `cargo clippy`         | Static analysis      |
| `cargo fmt -- --check` | Check formatting     |
| `cargo doc`            | Generate docs        |
| `cargo clean`          | Clean artifacts      |

<details>
<summary>Cross-Platform Builds</summary>

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
</details>

## Configuration

<details>
<summary>Environment Setup</summary>

```bash
OPENAI_API_KEY=key
SAMBASTUDIO_API_KEY=key
GEMINI_API_KEY=key
DATABASE_PATH=./harper.db
```
</details>

<details>
<summary>Advanced Config</summary>

```toml
[api]
timeout = 90
retry_attempts = 3

[cache]
enabled = true
ttl_seconds = 300
```
</details>

## Security

- Local SQLite storage
- No external data transmission
- Environment-based credentials
- AES-GCM-256 encryption
- CodeQL vulnerability detection
- DevSkim security scanning
- Dependency auditing
- Input validation



## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

<details>
<summary>Development Setup</summary>

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cargo fetch
cargo test
cargo clippy
cargo fmt -- --check
```
</details>

## Links

- [Issues](https://github.com/harpertoken/harper/issues)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [License](LICENSE)