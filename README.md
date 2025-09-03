# Harper

[![CI Status](https://github.com/harpertoken/harper/actions/workflows/ci.yml/badge.svg)](https://github.com/harpertoken/harper/actions)
[![Security Audit](https://github.com/harpertoken/harper/actions/workflows/security.yml/badge.svg)](https://github.com/harpertoken/harper/actions)
[![Release](https://img.shields.io/github/v/release/harpertoken/harper)](https://github.com/harpertoken/harper/releases)

A high-performance Rust-based AI agent for multi-provider integration, secure command execution, and advanced security analysis with local SQLite storage.

## Latest Release: v1.3.0

**Major Quality & Reliability Update** - Complete CodeQL & CI Build Resolution
- Resolved CodeQL dependency conflicts (20+ → 9 minor conflicts)
- Fixed CI build failures across all platforms (Linux, Windows, macOS)
- Enhanced security analysis with improved DevSkim integration
- Cross-platform compatibility verified on all supported architectures

## Requirements

* **Rust:** 1.70.0+ (MSRV verified)
* **Network:** Connectivity for API calls
* **Platform:** Linux, macOS, or Windows
* **Storage:** SQLite3 for data persistence

## Quick Start

### Installation

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/harpertoken/harper.git
cd harper

# Build release version
cargo build --release

# Configure environment
cp env.example .env
# Edit .env with your API keys

# Run
cargo run --release
```

### Alternative: Install from Release

```bash
# Install directly from GitHub release
cargo install --git https://github.com/harpertoken/harper.git --tag v1.3
```

## Usage

```text
[SEARCH: query]
[RUN_COMMAND command]
[TOOL: name] { "param": "value" }
```

## Key Features

### Multi-Provider AI Integration

| Provider  | Model                      | Capabilities              | Status |
|-----------|----------------------------|---------------------------|--------|
| OpenAI    | GPT-4 Turbo               | Text generation, coding   | Active |
| Sambanova | Meta-Llama-3.2-1B-Instruct| Open-source LLM           | Active |
| Gemini    | Gemini 2.0 Flash          | Multimodal processing     | Active |

### Advanced Capabilities

- Secure Command Execution - Safe shell command execution with validation
- Intelligent Web Search - Integrated web search functionality
- Session Management - Persistent conversation history with SQLite
- Multi-format Export - Export sessions in various formats
- Real-time Interaction - Interactive CLI with colored output

### Security & Quality

- CodeQL Integration - Advanced security vulnerability detection
- DevSkim Scanning - Automated security analysis
- Dependency Auditing - Regular security updates and checks
- Cryptographic Operations - AES-GCM-256, SHA-256, secure key management
- Input Validation - Comprehensive security validation

### Performance & Reliability

- Cross-Platform - Verified builds on Linux, Windows, macOS (Intel + ARM)
- CI/CD Pipeline - Automated testing across all platforms
- Memory Safe - Rust's memory safety guarantees
- Optimized Builds - Release builds with performance optimizations

### Model Context Protocol (MCP)

Note: MCP functionality is temporarily disabled in v1.3.0 to resolve dependency conflicts. It can be re-enabled with a compatible client version.

```toml
# Future MCP configuration (when re-enabled)
[mcp]
enabled = true
server_url = "http://localhost:5000"
```

### Data Management

- SQLite Storage - Local database for conversation history
- Session Persistence - Never lose your conversation context
- Export Capabilities - Save and share conversation sessions
- Secure Credentials - Local environment-based API key storage

## Build & Development

### Build Commands

| Command                                       | Description                      |
|-----------------------------------------------|----------------------------------|
| `cargo build --release`                       | Optimized release build          |
| `cargo run --release`                         | Execute release binary           |
| `cargo test --all-features --workspace`       | Run complete test suite          |
| `cargo fmt --all -- --check`                  | Verify code formatting           |
| `cargo clippy --all-targets --all-features`   | Static analysis & linting        |
| `cargo doc --open`                            | Generate and open documentation  |
| `cargo clean`                                 | Remove build artifacts           |
| `make build`                                  | Alternative build via Makefile   |

### Cross-Platform Building

```bash
# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Windows (x86_64)
cargo build --release --target x86_64-pc-windows-msvc

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin
```

## ⚙️ Configuration

### Environment Setup

Create `.env` file with your API keys:

```bash
# Required API Keys
OPENAI_API_KEY=your_openai_key_here
SAMBASTUDIO_API_KEY=your_sambanova_key_here
GEMINI_API_KEY=your_gemini_key_here

# Optional: Database path (defaults to local SQLite)
DATABASE_PATH=./harper.db
```

### Advanced Configuration

Edit `config/default.toml` for advanced settings:

```toml
[api]
timeout = 90
retry_attempts = 3

[cache]
enabled = true
ttl_seconds = 300

[logging]
level = "info"
format = "json"
```

## Security & Privacy

### Data Handling
- Local Storage Only - All data stored locally in SQLite database
- No Data Transmission - API keys and conversations never leave your machine
- Secure Credentials - Environment-based API key storage
- Encrypted Operations - AES-GCM-256 for sensitive data

### Security Features
- CodeQL Integration - Automated security vulnerability detection
- DevSkim Scanning - Static security analysis
- Dependency Auditing - Regular security updates
- Input Validation - Comprehensive security checks
- Memory Safety - Rust's compile-time memory safety guarantees

### Analysis & Scanning
- Static Analysis: Clippy with security-focused rules
- Security Scanning: DevSkim vulnerability detection
- Dependency Audit: Automated security vulnerability checks
- SARIF Integration: Security findings reported to GitHub Security tab
- Cross-Platform Verification: Security checks on all supported platforms

## Recent Updates (v1.3.0)

### Major Improvements
- Dependency Resolution: Fixed 20+ CodeQL conflicts → 9 minor conflicts
- CI/CD Reliability: All platforms building successfully
- Security Enhancement: Improved CodeQL and DevSkim integration
- Cross-Platform: Verified builds on Linux, Windows, macOS

### Technical Fixes
- Cargo.lock Sync: Resolved lock file synchronization issues
- Build Compatibility: Fixed `--locked` flag conflicts
- Security Scanning: Enhanced vulnerability detection
- Code Quality: Clippy clean with zero warnings

### Quality Metrics
- CI Checks: 15/15 passing across all platforms
- Security Audit: Clean results
- Test Coverage: Comprehensive test suite
- Performance: Optimized release builds

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
```bash
# Clone repository
git clone https://github.com/harpertoken/harper.git
cd harper

# Install dependencies
cargo fetch

# Run tests
cargo test --all-features

# Check code quality
cargo clippy --all-targets --all-features
cargo fmt --all -- --check
```

### Reporting Issues
- Bug Reports: [GitHub Issues](https://github.com/harpertoken/harper/issues)
- Security Issues: [Security Policy](SECURITY.md)
- Feature Requests: [Discussions](https://github.com/harpertoken/harper/discussions)

## License & Legal

License: Apache 2.0 - See [LICENSE](LICENSE) for details

Privacy: See [PRIVACY.md](PRIVACY.md) for our privacy policy

Security: See [SECURITY.md](SECURITY.md) for security information

## Acknowledgments

- Rust Community - For the excellent ecosystem and tools
- Open Source Contributors - For their valuable contributions
- Security Researchers - For helping improve our security posture

---

Built with Rust | Latest Release: v1.3.0 | License: Apache 2.0