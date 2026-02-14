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

## Install

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp config/env.example .env
cargo run --release
```

---

## Requirements

* Rust 1.85.0 or newer
* API key for at least one provider (OpenAI, Sambanova, or Gemini)
* Crossterm-compatible terminal

---

## Configuration

Harper uses TOML files and environment variables.

* Select AI provider and model
* Control command approval policy
* Choose UI theme
* Configure storage location

API keys should be provided via environment variables.

---

## Security

* No silent execution
* Scoped filesystem access
* Shell metachar filtering
* Environment-based credentials

---

## Architecture

```
harper-core        agents · tools · memory
harper-ui          terminal interface
harper-mcp-server  extensibility
```

---

## Troubleshooting / FAQ

**Commands do not run**
Check that execution approval is enabled and confirm prompts.

**API errors**
Verify API keys and provider quotas.

**UI rendering issues**
Ensure your terminal supports 24-bit color and try `TERM=xterm-256color`.

**Build failures**
Confirm your Rust toolchain meets the minimum version.

---

## License

Apache 2.0
