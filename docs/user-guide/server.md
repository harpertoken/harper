# HTTP API Server

Harper includes an optional HTTP API server mode for programmatic access.

## Enabling the Server

Add to your config (`config/local.toml`):

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8081
```

## Run both TUI and server

You can run the server in one terminal and the TUI in another:

```bash
# Terminal 1 - server + TUI
cargo run -p harper-ui --bin harper

# Terminal 2 - TUI only (no server)
cargo run -p harper-ui --bin harper -- --no-server
```

## API Endpoints

| Method | Endpoint | Description |
|-------|---------|-----------|
| GET | `/health` | Health check |
| GET | `/api/sessions` | List sessions |
| GET | `/api/sessions/{id}` | Get session messages |
| DELETE | `/api/sessions/{id}` | Delete session |
| POST | `/api/chat` | Send chat message |
| POST | `/api/review` | Review code and return inline findings |

## Test the server

```bash
# Start server
cargo run -p harper-ui --bin harper

# In another terminal
curl http://127.0.0.1:8081/health
# {"status":"ok","version":"0.10.0"}

curl http://127.0.0.1:8081/api/sessions
# {"sessions":[]}
```

## Review code

```bash
curl -X POST http://127.0.0.1:8081/api/review \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "lib/harper-core/src/server/mod.rs",
    "language": "rust",
    "max_findings": 5,
    "instructions": "Focus on correctness and concrete fixes.",
    "content": "fn main() { let x = 1; let x = 2; }"
  }'
```

Example response:

```json
{
  "summary": "Revised code to eliminate redundant variable declaration",
  "findings": [
    {
      "title": "Redundant Variable Declaration",
      "severity": "error",
      "message": "The second assignment to `x` is unnecessary...",
      "range": {
        "start_line": 1,
        "start_column": 13,
        "end_line": 1,
        "end_column": 24
      },
      "suggestion": {
        "description": "Remove the redundant `let x = 2;` line",
        "replacement": "fn main() { let x = 2; }"
      }
    }
  ],
  "model": "llama3"
}
```

## VS Code

A lightweight VS Code extension scaffold lives at `extensions/harper-review-vscode`. It calls `/api/review`, publishes diagnostics, and exposes quick fixes.

Run `Cmd+Shift+P` → "Harper: Review Current File", or add a custom shortcut:

```json
{
  "key": "cmd+shift+r",
  "command": "harperReview.reviewCurrentFile"
}
```

View findings in `Cmd+Shift+M` (Problems view).

### Verify diagnostics

After running "Harper: Review Current File", look for:

- **Status bar** - Bottom-right corner. You should see:
  - `$(warning) Harper N` - warning icon with **N findings** the LLM found in this file
  - `$(check) Harper Clean` - green check if no issues
  - `$(error) Harper Review` - red X if error

  The number (N) changes for each file - it's the actual count of issues detected, not a fixed value.

- **Problems view** - Press `Cmd+Shift+M`. Look for:
   - Source: "Harper Review"
   - Error/Warning/Info icons
   - File path and line number

- **Quick fixes** - In the editor:
   - Red/yellow/blue squiggly underlines
   - Hover to see the finding message
   - Click lightbulb or press `Cmd+.` for fix options

### Rebuild VSIX

After editing the extension, rebuild and reinstall:

```bash
cd extensions/harper-review-vscode
vsce package --out harper-review.vsix
```

Then in VS Code: **Extensions** → **...** → **Install from VSIX...**

## Troubleshooting

### Server not running

```bash
curl http://127.0.0.1:8081/health
# {"status":"ok","version":"0.10.0"}
```

If it fails, start the server: `cargo run -p harper-ui --bin harper`

### Extension shows "fetch failed"

- Check server is running: `curl http://127.0.0.1:8081/health`
- Reload VS Code: **Cmd+Shift+P** → "Developer: Reload Window"
- Check port in settings: **Cmd+,** → search "harperReview.serverUrl"

### Port already in use

If you get "Address already in use", stop existing server or use different port:

```bash
pkill -f harper
# Or change port in config/local.toml:
[server]
port = 8082
```

## Change port

To use a different port, update these files:

### Code (backend)
- `config/local.toml` - main config
- `lib/harper-ui/src/main.rs` - server fallback when no config port is set

### VS Code extension
- `extension.js` - server URL
- `package.json` - default serverUrl

### Docs
- `website/server.html` - curl commands
- `docs/user-guide/server.md` - curl commands
- `extensions/README.md` - config example

## Docker

In Docker, the server is enabled by default:

```bash
docker run --rm -it -p 8080:8080 harper
```
