# Harper AI Agent

[![CI](https://github.com/harpertoken/harper/actions/workflows/ci.yml/badge.svg)](https://github.com/harpertoken/harper/actions/workflows/ci.yml)
[![Release](https://github.com/harpertoken/harper/actions/workflows/release.yml/badge.svg)](https://github.com/harpertoken/harper/actions/workflows/release.yml)



**Harper AI Agent** is a Rust-based tool for connecting to multiple AI providers, executing shell commands, integrating with MCP (Model Context Protocol), and maintaining conversation history — all locally.

---

## System Requirements

* Rust 1.70.0 or later
* Internet connection
* Supported OS: Linux, macOS, Windows (WSL2 recommended)

---

## Quick Start

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build Harper
git clone https://github.com/harpertoken/harper.git
cd harper
make build

# Configure API keys
cp env.example .env
# Edit .env with your preferred API keys

# Run Harper
make run
```

**Example Usage**

```text
[SEARCH: rust crates]
[RUN_COMMAND echo "Hello, Harper!"]
[TOOL: tool_name] { "parameter": "value" }
```

---

## Core Features

### Multi-Provider AI Integration

| Provider      | Model                      | Best For                         |
| ------------- | -------------------------- | -------------------------------- |
| OpenAI        | GPT-4 Turbo                | General purpose, code generation |
| Sambanova     | Meta-Llama-3.2-1B-Instruct | Open-source alternative          |
| Google Gemini | Gemini 2.0 Flash           | Multimodal capabilities          |

### Model Context Protocol (MCP)

Harper supports MCP for enhanced tool integration. Enable in `config/default.toml` or `config/local.toml`:

```toml
[mcp]
enabled = true
server_url = "http://localhost:5000"
```

Run an MCP-compatible server (e.g., [Codex MCP](https://github.com/harpertoken/codex)):

```bash
git clone https://github.com/harpertoken/codex.git
cd codex
cargo run
```

Once running, Harper connects automatically. MCP features include tool discovery, resource access, and prompt generation.

### Command Execution & Web Search

* Run shell commands: `[RUN_COMMAND <command>]`
* Perform searches: `[SEARCH: <query>]`

### Cryptography & Security

* AES-GCM Encryption/Decryption (256-bit)
* SHA-256 Hashing
* Zero-Knowledge Proofs (via MCL)
* Nonce Management

Supports secure storage, encrypted messaging, and data integrity verification.

### Session Management

* Save/load conversation history
* Export conversations in multiple formats
* Persistent storage using SQLite

---

## Dependencies

* [MCL](https://github.com/herumi/mcl) — elliptic curve and cryptography library
* [MCP](https://modelcontextprotocol.io) — model context protocol

---

## Commands Reference

| Command                                                                    | Description                                            |
| -------------------------------------------------------------------------- | ------------------------------------------------------ |
| `make build`                                                               | Build release version                                  |
| `make run`                                                                 | Run Harper                                             |
| `cargo test --all-features --workspace --verbose`                          | Run all tests across the workspace                     |
| `cargo test --all-features --workspace --release --verbose`                | Run all tests in release mode                          |
| `rustup default 1.70.0 && cargo test --all-features --workspace --verbose` | Set Rust version to 1.70.0 and run all workspace tests |
| `make fmt`                                                                 | Format code locally                                    |
| `cargo fmt --all -- --check`                                               | Check code formatting (CI/verification)                |
| `make lint`                                                                | Run linter                                             |
| `cargo clippy --all-targets --all-features --workspace -- -D warnings`     | Run Clippy and treat all warnings as errors            |
| `make doc`                                                                 | Generate documentation                                 |
| `make clean`                                                               | Clean build artifacts                                  |
| `gh run list -w ci.yml --limit 1`                                          | Show the latest CI workflow run using GitHub CLI       |

---

## Configuration

Create a `.env` file:

```text
# Choose one provider
OPENAI_API_KEY=your_openai_key
SAMBASTUDIO_API_KEY=your_sambanova_key
GEMINI_API_KEY=your_gemini_key
```

---

## Privacy & Data Collection

**Harper AI Agent** respects your privacy and is designed with data protection in mind.

### Data Collection & Usage

We collect and process data to provide and improve our AI assistant service:

**Essential Data (Required for functionality):**
- API keys for AI provider authentication (stored locally only)
- Conversation history and session data (stored in local SQLite database)
- Configuration settings (stored locally in TOML files)

**Performance & Analytics Data:**
- Usage statistics to understand service performance
- Error logs for debugging and improvement
- Device identifiers for session management

**Data Sharing:**
- AI provider requests are sent directly to their respective APIs
- No conversation data is shared with third parties
- Usage analytics may be collected anonymously for service improvement

### Data Storage

- All conversation history is stored locally in a SQLite database
- API keys are stored in local environment files
- No data is transmitted to our servers unless explicitly configured

### Your Rights

You can:
- Delete your conversation history at any time
- Modify or delete stored API keys
- Opt out of analytics by not using the application
- Review and update your configuration settings

### Learn More

For more information about our data practices, see our [Privacy Policy](PRIVACY.md).

---

## Error Handling

* Syntax errors with exact locations
* Code quality checks via Clippy
* Detailed stack traces for test failures
* Security vulnerability scanning

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Documentation

For detailed documentation, patterns, and guides, visit our [Wiki](https://github.com/harpertoken/harper/wiki).

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).

---

## Community & Support

* GitHub Discussions: [Link](https://github.com/harpertoken/harper/discussions)
* Discord Server: [Link](https://discord.gg/ENUnDfjA)
* X: [Link](https://x.com/harper56889360)

Submit **bug reports** as issues or start a discussion for feature requests/questions.