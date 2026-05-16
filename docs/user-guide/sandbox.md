# Sandbox

Harper can run shell commands with a filesystem and network boundary.

## Modes

Use `sandbox_profile` under `[exec_policy]`:

```toml
[exec_policy]
sandbox_profile = "workspace"
```

| Mode | Behavior |
| --- | --- |
| `disabled` | Runs commands without a sandbox. |
| `workspace` | Allows workspace reads/writes, makes `$HOME` read-only, and blocks network by default. |
| `networked_workspace` | Allows workspace reads/writes, makes `$HOME` read-only, and allows network. |

Each command output starts with the active mode, for example:

```text
sandbox: workspace (bubblewrap (bwrap), network: off)
```

## Backends

| Platform | Backend |
| --- | --- |
| Linux | `bubblewrap` (`bwrap`) |
| macOS | `sandbox-exec` |
| Windows | Not supported yet |

If sandboxing is enabled and Harper cannot find a supported backend, the command fails instead of silently running unsandboxed.

## Custom policy

Profiles can be overridden with explicit paths:

```toml
[exec_policy.sandbox]
allowed_dirs = ["."]
writable_dirs = ["."]
network_access = false
readonly_home = true
max_execution_time_secs = 30
```

Use this only when the built-in profiles are too broad or too narrow.
