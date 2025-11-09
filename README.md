# Harper

[![Release](https://img.shields.io/github/v/release/harpertoken/harper)](https://github.com/harpertoken/harper/releases)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.82.0+-blue)](https://rust-lang.org)

AI agent for multi-provider integration, command execution, and MCP protocol support with SQLite storage.

Harper provides a unified interface to multiple AI providers (OpenAI, Sambanova, Gemini) with persistent chat sessions, command execution capabilities, and Model Context Protocol (MCP) support.

## Requirements

- **Rust**: 1.82.0 or later ([install Rust](https://rustup.rs/))
- **Operating System**: Linux, macOS, or Windows
- **Database**: SQLite3 (included with most systems)
- **Network**: Internet connectivity for AI provider APIs
- **Memory**: 512MB RAM minimum, 1GB recommended

## Installation

### Local Build

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and build**:
   ```bash
   git clone https://github.com/harpertoken/harper.git
   cd harper
   cargo build --release
   ```

3. **Configure environment**:
   ```bash
   cp config/env.example .env
   # Edit .env with your API keys (see Configuration section below)
   ```

4. **Run Harper**:
   ```bash
   cargo run --release
   ```

### Docker

Harper provides pre-built Docker images for easy deployment.

```bash
# Clone the repository
git clone https://github.com/harpertoken/harper.git
cd harper

# Copy and configure environment
cp config/env.example .env
# Edit .env with your API keys

# Build and run
docker build -t harper .
docker run --rm -it --env-file .env -v harper_data:/app/data harper
```

**Note**: Docker builds are validated in CI via GitHub Actions. For detailed Docker instructions, see [docker/DOCKER.md](docker/DOCKER.md).

## Development

### Running Tests

Harper includes a comprehensive test suite covering unit tests, integration tests, security, and performance benchmarks.

#### Quick Test Run

To run all tests, use the provided script:

```bash
./harpertest
```

This executes:
- **Unit tests**: Core functionality tests
- **Integration tests**: End-to-end API and database tests
- **Error handling tests**: Failure scenario validation
- **Security tests**: Input validation and encryption
- **Performance benchmarks**: Response time measurements

#### Example Output

```
Running all tests and benchmarks...
=================================

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
=================================
```

#### Test Coverage

The test suite includes:
- **12 unit tests** - Core component functionality
- **10 integration tests** - Full system workflows
- **6 error handling tests** - Edge cases and failures
- **3 security tests** - Encryption and validation
- **Performance benchmarks** - Speed and efficiency metrics

#### Individual Test Commands

```bash
# Run specific test types
cargo test --lib                    # Unit tests only
cargo test --test integration_test  # Integration tests
cargo test --test session_service_test  # Session service tests

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

<details>
<summary>Install from Release</summary>

```bash
# Check latest release at https://github.com/harpertoken/harper/releases
cargo install --git https://github.com/harpertoken/harper.git --tag <latest-tag>
```
</details>

<details>
<summary>CI/CD</summary>

Harper uses GitHub Actions for automated testing and deployment:

- **CI**: Comprehensive testing, linting, and security audits
- **Docker**: Builds, tests, and publishes Docker images
- **Title Checks**: Validates commit messages and PR titles
- **Release Drafter**: Generates changelogs and draft releases
- **Release**: Automated cross-platform binary builds and publishing

Workflows run on pushes and pull requests. See `.github/workflows/` for details.
</details>

## Conventional Commits

This project uses conventional commit standards to ensure consistent and meaningful commit messages.

### Setup

To enable commit message validation:

```bash
cp scripts/commit-msg .git/hooks/commit-msg
chmod +x .git/hooks/commit-msg
```

### Usage

Commit messages must follow the format: `type(scope): description`

- **Type**: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert
- **Scope**: Optional, e.g., (api), (ui)
- **Description**: Lowercase, ≤60 characters

Examples:
- `feat: add user authentication`
- `fix(ui): resolve button alignment issue`
- `docs: update installation instructions`

### History Cleanup

To rewrite existing commit messages in the history:

```bash
git filter-branch --msg-filter 'bash scripts/rewrite_msg.sh' -- --all
git push --force-with-lease
```

This will lowercase and truncate first lines to 60 characters.

## Usage

### Quick Start

After installation, run Harper and follow the interactive menu:

```bash
cargo run --release
# or
docker run --rm -it --env-file .env harper
```

Harper will display the configured AI provider and present a text-based menu for:
- Starting new chat sessions
- Managing conversation history
- Exporting sessions
- Running commands

### Interactive Mode

Harper runs in interactive mode by default, providing a menu-driven interface:

```
🤖 Using OpenAI - gpt-4-turbo
📍 API: https://api.openai.com/v1/chat/completions
💾 Database: ./chat_sessions.db

1. Start new chat session
2. List previous sessions
3. View a session's history
4. Export a session's history
5. Quit

Enter choice:
```

### AI Agent Commands

Within chat sessions, you can use special commands:

```
[SEARCH: query]           # Web search functionality
[RUN_COMMAND command]     # Execute system commands
[TOOL: name] { "param": "value" }  # Use MCP tools
```

## Features

### AI Providers

Harper supports multiple AI providers with automatic model selection:

| Provider  | Model | Capabilities | Status |
|-----------|-------|--------------|--------|
| **OpenAI** | GPT-4 Turbo | Text generation, coding, analysis | ✅ Production |
| **Sambanova** | Meta-Llama-3.2-1B-Instruct | Open-source LLM, cost-effective | ✅ Production |
| **Gemini** | Gemini 2.0 Flash | Multimodal processing, fast responses | ✅ Production |

### Core Capabilities

- **Multi-Provider AI Integration**: Seamless switching between AI providers
- **Command Execution**: Safe execution of system commands with output capture
- **Web Search**: Integrated search capabilities for real-time information
- **Persistent Sessions**: SQLite-based conversation history with full export
- **Interactive CLI**: User-friendly text-based interface
- **Session Management**: List, view, and export chat histories

### Security & Reliability

- **CodeQL Security Scanning**: Automated vulnerability detection
- **DevSkim Analysis**: Security-focused code review
- **Dependency Auditing**: Regular security updates
- **AES-GCM Encryption**: Secure data storage and transmission
- **Input Validation**: Comprehensive request sanitization
- **Error Handling**: Robust failure recovery and logging

### Model Context Protocol (MCP)

**Status**: Temporarily disabled in v0.1.3+ due to dependency conflicts.

When re-enabled, MCP provides:
- Tool integration capabilities
- External service connections
- Extended functionality through plugins

```toml
[mcp]
enabled = true
server_url = "http://localhost:5000"
```

### Data Management

- **SQLite Storage**: Lightweight, file-based database
- **Local Credentials**: No external account requirements
- **Session Persistence**: Automatic conversation saving
- **Export Functionality**: JSON/CSV export of chat histories
- **Backup Support**: Easy data migration and recovery

## Build Commands

### Basic Commands

| Command | Description |
|---------|-------------|
| `cargo build --release` | Optimized release build |
| `cargo run --release` | Run the release binary |
| `cargo test` | Execute test suite |
| `cargo clippy` | Run linting and static analysis |
| `cargo fmt -- --check` | Check code formatting |
| `cargo doc` | Generate documentation |
| `cargo clean` | Remove build artifacts |

### Development Workflow

```bash
# Full development cycle
cargo fmt                    # Format code
cargo clippy                 # Lint code
cargo test                   # Run tests
cargo build --release        # Build optimized binary
cargo run --release          # Run application
```

### Cross-Platform Builds

Harper can be built for multiple platforms:

```bash
# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Windows (x86_64)
cargo build --release --target x86_64-pc-windows-msvc

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

### Build Optimization

For maximum performance in production:

```bash
# Enable Link Time Optimization (LTO)
cargo build --release --config profile.release.lto=true

# Use specific optimization level
cargo build --release --config profile.release.opt-level=3
```

## Configuration

### Environment Variables

Harper uses environment variables for configuration. Set one of the following API keys:

```bash
# Choose your AI provider
export OPENAI_API_KEY="your-openai-key"
# OR
export SAMBASTUDIO_API_KEY="your-sambanova-key"
# OR
export GEMINI_API_KEY="your-gemini-key"

# Optional: Custom database location
export DATABASE_PATH="./harper.db"
```

### Model Selection

Harper automatically selects the appropriate model based on your API key:

| Environment Variable | Provider | Model | Base URL |
|---------------------|----------|-------|----------|
| `OPENAI_API_KEY` | OpenAI | `gpt-4-turbo` | `https://api.openai.com/v1/chat/completions` |
| `SAMBASTUDIO_API_KEY` | Sambanova | `Llama-4-Maverick-17B-128E-Instruct` | `https://api.sambanova.ai/v1/chat/completions` |
| `GEMINI_API_KEY` | Gemini | `gemini-2.0-flash-exp` | `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent` |

### Configuration Files

For advanced configuration, edit `config/local.toml`:

```toml
[api]
provider = "OpenAI"  # OpenAI, Sambanova, or Gemini
api_key = "your_api_key_here"
base_url = "https://api.openai.com/v1/chat/completions"
model_name = "gpt-4-turbo"

[database]
path = "./harper.db"

[mcp]
enabled = false
server_url = "http://localhost:5000"
```

### Configuration Priority

1. **Environment variables** (highest priority)
2. `config/local.toml` (overrides defaults)
3. `config/default.toml` (fallback defaults)




## Security

Harper implements multiple layers of security to protect your data and ensure safe operation:

### Data Protection

- **Local Storage Only**: All data stored locally in SQLite database
- **No External Transmission**: Conversations never leave your device
- **Environment-Based Credentials**: API keys stored in environment variables
- **AES-GCM-256 Encryption**: Secure encryption for sensitive data

### Code Security

- **CodeQL Scanning**: Automated vulnerability detection in CI/CD
- **DevSkim Analysis**: Security-focused static analysis
- **Dependency Auditing**: Regular security updates and checks
- **Input Validation**: Comprehensive request sanitization

### Operational Security

- **Command Sandboxing**: Safe command execution with restricted permissions
- **Error Handling**: Secure failure responses without information leakage
- **Logging Security**: Sensitive data redaction in logs

### Compliance

- **GDPR Compliant**: No personal data collection or transmission
- **Privacy-First**: All processing happens locally
- **Open Source**: Full transparency and community review

For security issues, please see our [Security Policy](SECURITY.md).

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for detailed information.

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/harpertoken/harper.git
   cd harper
   ```

2. **Install dependencies**:
   ```bash
   cargo fetch
   ```

3. **Run the test suite**:
   ```bash
   cargo test
   ```

4. **Code quality checks**:
   ```bash
   cargo clippy    # Linting
   cargo fmt -- --check  # Formatting check
   ```

5. **Build and test**:
   ```bash
   cargo build --release
   ./harpertest    # Run full test suite
   ```

### Development Workflow

- Follow [Conventional Commits](https://conventionalcommits.org/) for commit messages
- Run tests before submitting PRs
- Ensure code passes all CI checks
- Update documentation for new features

### Getting Help

- **Issues**: [GitHub Issues](https://github.com/harpertoken/harper/issues)
- **Discussions**: [GitHub Discussions](https://github.com/harpertoken/harper/discussions)
- **Documentation**: See [docs/](docs/) directory

### Code of Conduct

This project follows a code of conduct to ensure a welcoming environment for all contributors. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Project Status

Harper is actively maintained and under continuous development. Current focus areas:

- **Performance Optimization**: Reducing latency and improving response times
- **MCP Protocol**: Re-enabling Model Context Protocol support
- **Additional Providers**: Expanding AI provider integrations
- **Enhanced Security**: Ongoing security improvements and audits

### Roadmap

- [ ] Web interface for chat sessions
- [ ] Plugin system for custom tools
- [ ] Multi-language support
- [ ] Advanced session analytics
- [ ] Cloud deployment options

## Acknowledgments

Harper builds upon the excellent work of the open-source community:

- **Rust Ecosystem**: For the robust systems programming language
- **SQLite**: For reliable, embedded database functionality
- **AI Providers**: OpenAI, Sambanova, and Google for accessible AI APIs
- **Contributors**: The community driving Harper's development

## Links

- [GitHub Repository](https://github.com/harpertoken/harper)
- [Issues](https://github.com/harpertoken/harper/issues)
- [Discussions](https://github.com/harpertoken/harper/discussions)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [License](LICENSE)