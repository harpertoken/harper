## Harper — Secure Configuration & Operation Guide

Harper is an AI agent that provides a unified interface to multiple AI providers (OpenAI, Sambanova, Gemini), with persistent chat sessions, command execution, MCP support, and SQLite-backed storage.

This guide describes **practical, low-friction configuration choices** that help keep Harper deployments safe, predictable, and easy to operate.

---

## Scope

This document focuses on:

* Sensible defaults for credential handling
* Guardrails around command and file operations
* Clear, user-controlled behavior for potentially sensitive actions

No changes are required to core functionality, and all guidance is compatible with existing workflows.

---

## Common Areas to Configure Thoughtfully

The following areas benefit from explicit configuration to ensure predictable behavior:

* **API credentials** — Prevent accidental disclosure
* **Command execution** — Keep system access intentional
* **File operations** — Avoid unintended file access
* **Session handling** — Maintain isolation between runs

These are not flaws, but natural considerations for tools that interact with external systems.

---

## Recommended Configuration Practices

### API Credentials

* Use **environment variables** for all API keys
* Keep configuration files free of real secrets
* Ensure `.env` files are excluded from version control

This keeps credentials local to the runtime environment and easy to rotate.

---

### Command Execution

* Require **explicit user approval** before running commands
* Reject shell metacharacters commonly associated with chaining or injection
* Keep execution scoped to the project workspace

This ensures commands are always intentional and observable.

---

### File Operations

* Validate file paths before access
* Default to the project directory
* Ask for confirmation on write operations

These checks reduce surprises while preserving flexibility.

---

### Session Behavior

* Keep sessions isolated
* Avoid sharing sensitive context across runs
* Prefer short-lived or encrypted storage for session data

This supports safe reuse without hidden coupling.

---

## UI Configuration

Harper includes a configurable Terminal UI (TUI) with theme support.

```toml
[ui]
theme = "dark" # default | dark | light
```

Themes affect:

* Background and foreground colors
* Message roles
* Borders and status indicators

UI customization is purely cosmetic and does not affect security behavior.

---

## Installation (Calm Path)

### Local Setup

```bash
git clone https://github.com/harpertoken/harper.git
cd harper

cp config/env.example .env
# add API keys locally

cargo build --release
cargo run --release
```

### Docker Setup

```bash
docker build -t harper .
docker run --rm -it \
  --env-file .env \
  --read-only \
  --tmpfs /tmp \
  harper
```

These options favor clarity and containment without introducing operational complexity.

---

## Implementation Notes (For Maintainers)

Harper prioritizes environment-based configuration for sensitive values and applies lightweight validation at runtime.
