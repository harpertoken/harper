# HTTP API Server

Harper includes an optional HTTP API server mode for programmatic access.

## Enabling the Server

Add to your config (`config/local.toml`):

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8080
```

## API Endpoints

| Method | Endpoint | Description |
|-------|---------|-----------|
| GET | `/health` | Health check |
| GET | `/api/sessions` | List sessions |
| GET | `/api/sessions/:id` | Get session messages |
| DELETE | `/api/sessions/:id` | Delete session |
| POST | `/api/chat` | Send chat message |

## Example

```bash
# Start server
cargo run -p harper-ui --bin harper

# In another terminal
curl http://127.0.0.1:8080/health
# {"status":"ok","version":"0.8.0"}

curl http://127.0.0.1:8080/api/sessions
```

## Docker

In Docker, the server is enabled by default:

```bash
docker run --rm -it -p 8080:8080 harper
```
