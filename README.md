```text
 _   _
| | | | __ _ _ __ _ __   ___ _ __
| |_| |/ _` | '__| '_ \ / _ \ '__|
|  _  | (_| | |  | |_) |  __/ |
|_| |_|\__,_|_|  | .__/ \___|_|
                  |_|
```

# Harper

**AI for the terminal.**

Harper is a terminal-native AI agent that translates natural language into reviewed, executable commands.

<details>
<summary>Show TUI preview</summary>

![Harper TUI](./docs/harper_tui.png)

</details>

&copy; 2026 harpertoken

---

## What it does

* Intent → shell commands
* Persistent context
* Explicit execution approval
* Terminal-only workflow

---

## Why

The shell is exact.
Humans are not.

Harper sits between them.

---

## Quick Start

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp .env.example .env
# Add your API key to .env
cargo run -p harper-ui --bin harper
```

Or after building:

```bash
cargo build --release
./target/release/harper
```

---

## Requirements

* Rust 1.85.0 or newer
* API key for at least one provider (OpenAI, Sambanova, or Gemini)
* Crossterm-compatible terminal

---

## Usage

Type what you want:

```
find rust files changed today
show uncommitted git changes
create a new feature branch
```

Harper shows the command.
You decide if it runs.

Need to review what ran? Type `/audit` (or `/audit 25 failed approved`) any time to print the latest shell commands filtered by limit/status/approval, with exit codes and runtimes tied to the current session.

---

## Providers

OpenAI · Sambanova · Gemini

(OpenAI-compatible endpoints supported)

---

## Configuration

Harper uses TOML files and environment variables.

* Select AI provider and model
* Control command approval policy
* Choose UI theme
* Configure storage location

API keys should be provided via environment variables.

See [docs/user-guide/configuration.md](docs/user-guide/configuration.md) for detailed config options.

---

## Documentation

See [docs/](docs/) for detailed project documentation.

### User Guide

* [Installation](docs/user-guide/installation.md) - Install Harper
* [Quick Start](docs/user-guide/quick-start.md) - Get started in 5 minutes
* [About the Binary](docs/user-guide/about.md) - How Harper runs
* [Chat Interface](docs/user-guide/chat.md) - Commands and features
* [Clipboard](docs/user-guide/clipboard.md) - Image and text processing
* [Configuration](docs/user-guide/configuration.md) - Complete config options
* [Troubleshooting](docs/user-guide/troubleshooting.md) - Common issues

---

## Security

* No silent execution
* Scoped filesystem access
* Shell metachar filtering
* Environment-based credentials

---

## Development

See Quick Start above for running Harper.

## Building

### Cargo (Recommended)

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build |
| `cargo build --release` | Release build |
| `cargo run -p harper-ui --bin harper` | Run in dev mode |
| `cargo test` | Run tests |

### Bazel

| Command | Description |
|---------|-------------|
| `bazel build //...` | Build all targets |
| `bazel run //:harper` | Run harper |
| `bazel test //...` | Run all tests |

Both build systems are supported. Cargo is recommended for development.

---

## Architecture

```
harper-core        agents · tools · memory
harper-ui          terminal interface
harper-mcp-server  extensibility
```

### Project Structure

```
.
├── .github/
│   ├── ISSUE_TEMPLATE/          # Issue templates
│   └── workflows/               # GitHub Actions
├── lib/
│   ├── harper-core/             # Core AI logic
│   ├── harper-ui/               # Terminal interface
│   └── harper-mcp-server/       # MCP server
├── docs/
│   ├── user-guide/              # User documentation
│   ├── development/             # Developer docs
│   └── getting-started/         # Getting started guides
├── scripts/                      # Utility scripts
├── config/                      # Configuration examples
├── tests/                       # Test files
├── Cargo.toml                    # Workspace manifest
└── README.md                    # This file
```

### Key Components

* `lib/harper-core/` - AI agents, tools, memory/persistence
* `lib/harper-ui/` - Terminal UI, command parsing, session management
* `lib/harper-mcp-server/` - MCP protocol server for extensibility

---

## Troubleshooting

**Commands do not run**
Check that execution approval is enabled and confirm prompts.

**API errors**
Verify API keys and provider quotas.

**UI rendering issues**
Ensure your terminal supports 24-bit color and try `TERM=xterm-256color`.

**Build failures**
Confirm your Rust toolchain meets the minimum version (1.85.0).

See [docs/user-guide/troubleshooting.md](docs/user-guide/troubleshooting.md) for more solutions.

---

## Features

- Natural language to shell command translation
- Persistent chat sessions
- Command approval workflow with audit trail
- Clipboard integration (images and text)
- Multiple AI provider support
- Session saving and loading
- Command history with search

---

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

See [LICENSE](LICENSE) for details.
