# Sandbox Isolation

Harper includes sandbox isolation for secure command execution.

## Supported Backends

- **Linux**: bubblewrap (bwrap)
- **macOS**: sandbox-exec

## Enabling Sandbox

Add to your config (`config/local.toml`):

```toml
[exec_policy.sandbox]
enabled = true
allowed_dirs = ["/app"]
network_access = false
readonly_home = true
max_execution_time_secs = 30
```

## Configuration Options

| Option | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | false | Enable sandbox |
| allowed_dirs | list | [] | Directories to allow |
| network_access | bool | false | Allow network access |
| readonly_home | bool | true | Read-only home |
| max_execution_time_secs | int | 30 | Command timeout |

## Docker

In Docker, sandbox is enabled by default with bubblewrap.

## Security

- Commands run in isolated namespace
- Filesystem access restricted to allowed directories
- Network disabled by default
- Commands timeout after 30 seconds
