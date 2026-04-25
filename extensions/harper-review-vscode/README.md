# Harper Review for VS Code

This extension connects VS Code to a local Harper server and turns Harper review findings into:

- inline diagnostics
- quick-fix suggestions
- apply-all fixes for the active file
- optional review-on-save

## Start Harper server

Enable the HTTP server in `config/local.toml`:

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8081
```

Then start Harper:

```bash
# Terminal 1 - server + TUI
cargo run -p harper-ui --bin harper

# Terminal 2 - TUI only (no server)
cargo run -p harper-ui --bin harper -- --no-server
```

## Load the extension

Open this folder in VS Code and use `Developer: Install Extension from Location...`, then select:

`extensions/harper-review-vscode`

## Commands

- `Harper: Review Current File`
- `Harper: Review Selection`
- `Harper: Apply All Suggested Fixes`
- `Harper: Clear Review Diagnostics`

Run in VS Code:
- Press **Cmd+Shift+P** → "Harper: Review Current File"
- Check **status bar** (bottom-right):
  - `$(warning) Harper N` - warning icon with **N findings** the LLM found in this file
  - `$(check) Harper Clean` - no issues found
  - `$(error) Harper Review` - error occurred

  The number (N) changes for each file - it's the actual count of issues detected.
- View findings: **Cmd+Shift+M** (Problems view)
- Quick fixes: Click lightbulb or **Cmd+.** on underlined code

## Rebuild VSIX

After editing the extension, rebuild the package:

```bash
cd extensions/harper-review-vscode
vsce package --out harper-review.vsix
```

Then reinstall in VS Code: **Extensions** → **...** → **Install from VSIX...**

Run via **Cmd+Shift+P** (Command Palette) or bind custom shortcuts in VS Code keybindings:

```json
{
  "key": "cmd+shift+r",
  "command": "harperReview.reviewCurrentFile"
}
```

## Settings

- `harperReview.serverUrl`
- `harperReview.autoReviewOnSave`
- `harperReview.maxFindings`
- `harperReview.instructions`
- `harperReview.requestTimeoutMs`

## Test the API

```bash
curl -X POST http://127.0.0.1:8081/api/review \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "test.rs",
    "language": "rust",
    "content": "fn main() { let x = 1; let x = 2; }"
  }'
```

Example real response:

```json
{
  "summary": "Revised code to eliminate redundant variable declaration",
  "findings": [
    {
      "title": "Redundant Variable Declaration",
      "severity": "error",
      "message": "The second assignment to `x` is unnecessary...",
      "range": {"start_line":1,"start_column":13,"end_line":1,"end_column":24},
      "suggestion": {"description":"Remove the redundant `let x = 2;`", "replacement":"fn main() { let x = 2; }"}
    }
  ],
  "model": "llama3"
}
```

## Review API

The extension sends `POST /api/review` with file content and optional selection metadata, then maps the response to VS Code diagnostics and quick fixes.
