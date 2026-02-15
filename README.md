# Welcome to Harper

Harper is an AI-powered terminal assistant that translates natural language into shell commands. Instead of remembering complex CLI syntax, just describe what you want to do in plain English.

```
find rust files changed today
show uncommitted git changes
create a new feature branch
```

Harper shows you the command. You decide if it runs.

---

![Harper TUI](./docs/harper_tui.png)

---

## How it works

Just type what you want. Harper understands your intent and generates the exact shell command you need. Review it, approve it, and Harper executes it for you. No more copy-pasting from AI chatbots or manually constructing complex commands.

## Key Features

**Natural Language to Commands**
Describe what you want in plain English and Harper generates the exact shell command. From simple file searches to complex git operations, just say what you need.

**Persistent Context**
Harper remembers your conversation, so you can build on previous commands and queries. Ask follow-up questions or reference earlier commands without repeating yourself.

**Explicit Execution Approval**
Never worry about accidental execution. Harper always shows you the command first and asks before running anything. You stay in complete control.

**Command Audit Trail**
Need to review what ran? Type `/audit` (or `/audit 25 failed approved`) to print the latest shell commands filtered by limit/status/approval, with exit codes and runtimes tied to the current session.

**Multiple AI Providers**
Choose from OpenAI, Sambanova, or Gemini. Harper also supports any OpenAI-compatible endpoint, giving you flexibility in how you power the AI.

## Contributor License Agreement

All contributors must sign the CLA before their pull requests can be merged.

```
❌ **CLA required**

Hi @username, please sign the Contributor License Agreement to proceed.

👉 https://harpertoken.github.io/cla.html

Once signed, this check will pass automatically.
```

When you submit a pull request, the CLA bot will check if your GitHub username is listed in the approved contributors list. If your name is not on the list, the pull request will be blocked until you are added. To have your username added, please contact the project owner at harpertoken@icloud.com.

## Getting Started

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

## Requirements

- Rust 1.85.0 or newer
- API key for at least one provider (OpenAI, Sambanova, or Gemini)
- Crossterm-compatible terminal

## Security

- No silent execution — commands always require approval
- Scoped filesystem access — Harper only touches what you ask it to
- Shell metachar filtering — prevents injection attacks
- Environment-based credentials — API keys stay in your environment

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

## Documentation

See [docs/](docs/) for detailed project documentation.

- [Installation](docs/user-guide/installation.md)
- [Quick Start](docs/user-guide/quick-start.md)
- [About the Binary](docs/user-guide/about.md)
- [Chat Interface](docs/user-guide/chat.md)
- [Clipboard](docs/user-guide/clipboard.md)
- [Configuration](docs/user-guide/configuration.md)
- [Config Reference](docs/CONFIG_REFERENCE.md)
- [Troubleshooting](docs/user-guide/troubleshooting.md)

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

See [LICENSE](LICENSE) for details.
