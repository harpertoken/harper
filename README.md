# Harper — AI-Powered Terminal Assistant

Harper is an AI agent that provides a unified interface to multiple AI providers (OpenAI, Sambanova, Gemini), with persistent chat sessions, command execution, MCP support, and SQLite-backed storage.

![Harper TUI](./docs/harper_tui.png)

---

## Introduction

The Command Line Interface (CLI) is one of the most powerful tools for interacting with computers. It lets you do almost anything—but it requires precise commands and deep knowledge of the shell's language.

Large Language Models (LLMs) trained on code have changed this. They understand both natural language and code well enough to translate between them. Harper brings this capability to your terminal.

With Harper, you can:
- **Ask in plain English** — "find all TypeScript files modified this week"
- **Get executable commands** — Harper translates your intent into shell commands
- **Chat persistently** — Sessions are saved to SQLite for continuity
- **Execute safely** — Guardrails, approval prompts, and validation protect your system

Harper is designed for developers who want AI assistance without leaving the terminal.

---

## Features

| Feature | Description |
|---------|-------------|
| **Multiple AI Providers** | OpenAI, Sambanova, Gemini — choose your backend |
| **Persistent Sessions** | SQLite-backed chat history across runs |
| **20+ Built-in Tools** | File ops, shell commands, git, web search, database queries |
| **MCP Support** | Extend capabilities via Model Context Protocol |
| **Secure by Default** | Approval prompts, path validation, shell metachar filtering |
| **Terminal UI** | Rich TUI with themes and syntax highlighting |

---

## Requirements

- **Rust 1.85.0+** — Build from source
- **API Keys** — OpenAI, Sambanova, or Gemini
- **Terminal** — Crossterm-compatible (most modern terminals)

---

## Installation

### From Source

```bash
git clone https://github.com/harpertoken/harper.git
cd harper

# Configure API keys
cp config/env.example .env
# Edit .env with your API credentials

# Build and run
cargo build --release
cargo run --release --bin harper
```

### Docker

```bash
docker build -t harper .
docker run --rm -it \
  -v "$(pwd)/data:/app/data" \
  --env-file .env \
  --read-only \
  --tmpfs /tmp \
  harper
```

---

## Usage

Start Harper with:

```bash
./bin/harper
```

### Quick Start

1. **Ask naturally** — Type what you want: "show me uncommitted git changes"
2. **Review suggestion** — Harper shows the command it would run
3. **Approve or edit** — Run as-is, modify, or reject
4. **Continue the conversation** — Harper remembers context within sessions

### Example Session

```
You: list all rust files in src/
Harper: find . -name "*.rs" -type f

You: how many lines does the main file have?
Harper: wc -l src/main.rs

You: create a new git branch for my feature
Harper: git checkout -b feature/my-new-feature
```

### Configuration

Harper uses TOML configuration. Key settings in `config/default.toml`:

```toml
[ai]
provider = "openai"  # openai | sambanova | gemini
model = "gpt-4"

[security]
require_approval = true   # Prompt before running commands
reject_metachars = true   # Block shell injection patterns

[ui]
theme = "dark"  # dark | light
```

Environment variables (recommended for API keys):

```bash
export OPENAI_API_KEY="sk-..."
export SAMBANOVA_API_KEY="..."
export GEMINI_API_KEY="..."
```

---

## Commands

Harper accepts natural language. Common patterns include:

| Pattern | Example |
|---------|---------|
| File operations | "read all JSON files in config/" |
| Git actions | "show diff of uncommitted changes" |
| Shell execution | "restart postgres service" |
| Web search | "find recent Rust TUI libraries" |
| Database queries | "list all tables in the users database" |
| Task management | "add 'review PR' to my todo list" |

Prefix commands with `#` for direct execution without AI translation.

---

## Architecture

### Core Components

```
lib/harper-core/     # AI logic, tools, memory management
lib/harper-ui/       # Terminal UI (Ratatui-based)
lib/harper-mcp-server/  # MCP protocol server
```

### Supported Tools

| Category | Tools |
|----------|-------|
| Filesystem | read, write, search, glob |
| Shell | execute commands with policy controls |
| Git | status, diff, commit, branch |
| Web | search, fetch content |
| Database | SQL query execution |
| Images | info, resize |
| Tasks | todo list management |
| MCP | external tool integration |

---

## Security

Harper is designed with security in mind:

- **Environment-based credentials** — API keys never in config files
- **Explicit approval** — Commands require user confirmation
- **Shell metachar filtering** — Blocks injection patterns
- **Path validation** — Scoped to project by default
- **Session isolation** — Encrypted storage available
- **Docker hardening** — Supports `--read-only` and `--tmpfs`

---

## Troubleshooting

### API Errors
Verify your API keys in `.env` or environment variables. Check billing/quotas on your provider's dashboard.

### TUI Rendering Issues
Ensure your terminal supports truecolor (24-bit) colors. Try `TERM=xterm-256color`.

### Command Not Running
Check that `require_approval` is set appropriately and you're not in restricted mode.

### Build Failures
Ensure Rust 1.85.0+ is installed: `rustc --version`

---

## FAQ

### Which AI providers are supported?
OpenAI (GPT-4, GPT-3.5), Sambanova, and Gemini are currently supported.

### Can I use my own model?
Yes. Configure any OpenAI-compatible API endpoint in `config/default.toml`.

### How does session storage work?
Sessions are stored in `data/` as SQLite databases. Set `HARPER_SESSION_DIR` to customize.

### Is my data sent to third parties?
Only to your configured AI provider. Harper does not collect or share data.

### Can I extend Harper with custom tools?
Yes. Use the MCP (Model Context Protocol) integration or contribute to `lib/harper-core/src/tools/`.

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

---

## License

Apache License 2.0 — see [LICENSE](./LICENSE) for details.
