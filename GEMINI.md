# Project Context: Harper

Harper is an AI-powered terminal assistant. This file provides key constraints and architectural details that Gemini CLI must adhere to when assisting with this codebase.

## Internal System Limits
- **Active History**: Harper's internal `ChatService` trims history to a maximum of **50 messages**.
- **API Timeout**: All LLM calls within Harper are hardcoded with a **90-second** timeout (`timeouts::API_REQUEST`).
- **Session Auto-Save**: Harper's configuration defaults to an auto-save interval of **300 seconds** (5 minutes).

## Licensing & Distribution
- **License**: Harper uses a proprietary **Commercial License**. A valid license key or subscription is required for commercial use.
- **Redistribution**: Reverse engineering or unauthorized redistribution of the binary is prohibited.

## Tech Stack
- **Language**: Rust (Minimum version 1.85.0).
- **Architecture**: Workspace-based (core, UI, and MCP server).
- **Providers**: Integrates with OpenAI, Sambanova, and Gemini APIs.

## User Interface
- **Theme Configuration**: Harper's theme is set at startup via the `[ui]` section in `config/local.toml` or the `HARPER_UI_THEME` environment variable.
- **Available Themes**: `default`, `dark`, `light`, and `github`.
- **Runtime Changes**: Harper does not support runtime theme switching; the application must be restarted to apply theme changes.

## Pull Request Description Guidelines

When generating a pull request description, follow this structure.

### Why

Explain the motivation behind the change.

Focus on:
- What was noticed while working
- What problem or friction existed
- Why the change was considered necessary

Write naturally and keep it concise.

### What Changed

Describe the modifications clearly.

Use short bullet points that explain:
- Files or components affected
- Behavior or UI changes
- Refactors, removals, or simplifications

Avoid repeating the commit history.

### Verification

Explain how the change was validated.

Include:
- How the change was tested
- Steps a reviewer can follow
- The expected result after applying the change

### Style

PR descriptions should read like a developer explaining their thought process
to a teammate. Prefer clarity over formality. Keep descriptions concise and
avoid unnecessary repetition.

## Troubleshooting

### Version Mismatch (Old UI/Binary)
If Harper is running an older version than expected or showing an outdated UI:
1. **Rebuild the project:** `make build`
2. **Update the local binary:** `cp target/release/harper bin/harper`
3. **Update the system binary:** `cp target/release/harper ~/.local/bin/harper` (or your platform's equivalent)
4. **Verify the version:** `harper --version`
