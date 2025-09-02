# Harper

A Rust-based AI agent for multi-provider integration, command execution, and MCP protocol support with local SQLite storage.

## Requirements

* Rust 1.70.0+
* Network connectivity for API calls
* Linux, macOS, or Windows (WSL2 recommended)

## Installation

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build project
git clone https://github.com/harpertoken/harper.git
cd harper
make build

# Configure environment
cp env.example .env
# Set API keys in .env file

# Execute
make run
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

### Model Context Protocol

MCP integration for tool discovery and resource access. Configuration in `config/default.toml`:

```toml
[mcp]
enabled = true
server_url = "http://localhost:5000"
```

### Command Execution

Shell command execution via `[RUN_COMMAND <command>]` syntax.

### Web Search

Web search functionality via `[SEARCH: <query>]` syntax.

### Cryptographic Operations

* AES-GCM-256 encryption/decryption
* SHA-256 hashing
* MCL-based zero-knowledge proofs
* Cryptographic nonce management

### Data Persistence

SQLite-based storage for conversation history and session data.

## Dependencies

* [MCL](https://github.com/herumi/mcl) - Elliptic curve cryptography
* [MCP](https://modelcontextprotocol.io) - Model Context Protocol

## Build Commands

| Command                                       | Function                          |
|-----------------------------------------------|-----------------------------------|
| `make build`                                  | Release build                     |
| `make run`                                    | Execute binary                    |
| `cargo test --all-features --workspace`       | Run test suite                    |
| `cargo fmt --all -- --check`                  | Verify code formatting            |
| `cargo clippy --all-targets --all-features`   | Static analysis                   |
| `make doc`                                    | Generate documentation            |
| `make clean`                                  | Remove build artifacts            |

## Configuration

Environment variables in `.env`:

```bash
OPENAI_API_KEY=key
SAMBASTUDIO_API_KEY=key
GEMINI_API_KEY=key
```

## Data Handling

### Storage
- Conversation history: Local SQLite database
- API credentials: Local environment file
- Configuration: Local TOML files

### Network Transmission
- API requests sent directly to provider endpoints
- No data transmitted to third-party servers
- All processing occurs locally

## Analysis

* **Static Analysis**: Clippy linting with security rules
* **Security Scanning**: DevSkim vulnerability detection
* **Error Reporting**: Syntax errors with line/column positions
* **SARIF Integration**: Security findings to GitHub Security tab

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md)

## License

Apache 2.0 - [LICENSE](LICENSE)