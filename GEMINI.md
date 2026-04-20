# Project Context: Harper

Harper is an AI-powered terminal assistant. This file provides key constraints and architectural details that Gemini CLI must adhere to when assisting with this codebase.

# Internal System Limits

Harper's internal `ChatService` trims history to a maximum of 50 messages. All LLM calls within Harper are hardcoded with a 90-second timeout. Harper's configuration defaults to an auto-save interval of 300 seconds (5 minutes).

# Licensing & Distribution

Harper uses a proprietary Commercial License. A valid license key or subscription is required for commercial use. Reverse engineering or unauthorized redistribution of the binary is prohibited.

# Tech Stack

Harper is written in Rust (minimum version 1.85.0) using a workspace-based architecture with multiple crates:

- `harper-core` provides the core logic.
- `harper-ui` provides the terminal user interface.
- `harper-mcp-server` provides the MCP server implementation.
- `harper-firmware` provides firmware abstraction for embedded devices like ESP32, STM32, and Raspberry Pi.
- `harper-sandbox` provides sandbox isolation using bubblewrap on Linux or sandbox-exec on macOS.

Harper integrates with OpenAI, Sambanova, and Gemini APIs.

# User Interface

Harper's theme is set at startup via the `ui` section in `config/local.toml` or the `HARPER_UI_THEME` environment variable. Available themes are `default`, `dark`, `light`, and `github`. Harper does not support runtime theme switching, so the application must be restarted to apply theme changes.

# Pull Request Description Guidelines

When generating a pull request description, explain the motivation behind the change. Focus on what was noticed while working, what problem or friction existed, and why the change was considered necessary. Write naturally and keep it concise.

Describe the modifications clearly using short bullet points that explain the files or components affected, behavior or UI changes, and any refactors, removals, or simplifications. Avoid repeating the commit history.

Explain how the change was validated by including how the change was tested, steps a reviewer can follow, and the expected result after applying the change.

PR descriptions should read like a developer explaining their thought process to a teammate. Prefer clarity over formality. Keep descriptions concise and avoid unnecessary repetition.

# Troubleshooting

If Harper is running an older version than expected or showing an outdated UI, rebuild the project with `make build`, update the local binary by copying `target/release/harper` to `bin/harper`, update the system binary by copying to `~/.local/bin/harper` (or your platform's equivalent), and verify the version by running `harper --version`.

# New Features

Harper introduced several new features.

The HTTP API Server allows running Harper as an HTTP server for programmatic access. Enable it in config with `server enabled = true` and set the port (default 8080). Available endpoints are `/health`, `/api/sessions`, `/api/sessions/:id`, and `/api/chat`.

Sandbox Isolation provides secure shell command execution using bubblewrap on Linux or sandbox-exec on macOS. Enable it in config with `exec_policy.sandbox enabled = true`. This restricts filesystem access to allowed directories and can disable network access.

Firmware Abstraction allows controlling embedded devices via chat commands like `[FIRMWARE list]`, `[FIRMWARE gpio 2 high]`, or `[FIRMWARE i2c ...]`. Supported platforms are ESP32, STM32, and Raspberry Pi Pico.
