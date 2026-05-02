# Harper Configuration Reference

This document is **authoritative**.
All unspecified values resolve to defaults.

---

## Configuration File

**Format:** TOML
**Default path:** `~/.harper/config.toml`
**Override:** `HARPER_CONFIG_PATH`

---

## `[provider]`

Controls AI backend selection and request behavior.

| Key           | Type    | Required | Default          | Description                       |
| ------------- | ------- | -------- | ---------------- | --------------------------------- |
| `name`        | string  | yes      | none             | Provider identifier               |
| `model`       | string  | yes      | none             | Model name                        |
| `endpoint`    | string  | no       | provider default | Custom OpenAI-compatible endpoint |
| `timeout_ms`  | integer | no       | 30000            | Request timeout                   |
| `max_retries` | integer | no       | 0                | Retry count on failure            |

### Allowed `name` values

| Value       | Notes               |
| ----------- | ------------------- |
| `openai`    | Native              |
| `sambanova` | OpenAI-compatible   |
| `gemini`    | Native              |
| `custom`    | Requires `endpoint` |

---

## `[execution]`

Command execution rules.
All rules are **deny-by-default**.

| Key                  | Type | Default | Effect                 |
| -------------------- | ---- | ------- | ---------------------- |
| `require_approval`   | bool | true    | Blocks auto-execution  |
| `allow_pipes`        | bool | false   | Enables `|`            |
| `allow_redirects`    | bool | false   | Enables `>`, `>>`, `<` |
| `allow_subshells`    | bool | false   | Enables `$()`          |
| `allow_background`   | bool | false   | Enables `&`            |
| `allow_sudo`         | bool | false   | Enables `sudo`         |
| `max_command_length` | int  | 4096    | Hard limit             |

### Hard-blocked (cannot be enabled)

| Pattern                         |
| ------------------------------- |
| Remote shell listeners          |
| Daemonized processes            |
| Privilege escalation via config |

---

## `[ui]`

Terminal UI behavior.

| Key               | Type   | Default    | Description            |
| ----------------- | ------ | ---------- | ---------------------- |
| `theme`           | string | `minimal`  | Minimal terminal theme |
| `show_exit_codes` | bool   | true       | Show status after run  |
| `confirm_style`   | string | `explicit` | `explicit` / `compact` |
| `vim_keys`        | bool   | true       | Enable j/k navigation  |

---

## `[storage]`

Session persistence and audit logs.

| Key            | Type   | Default              | Description       |
| -------------- | ------ | -------------------- | ----------------- |
| `path`         | string | `~/.harper/sessions` | Storage directory |
| `persist`      | bool   | true                 | Save sessions     |
| `max_sessions` | int    | 100                  | Retention limit   |
| `compress`     | bool   | false                | Gzip logs         |

---

## `[security]`

Additional constraints.

| Key                   | Type | Default | Description                      |
| --------------------- | ---- | ------- | -------------------------------- |
| `redact_env`          | bool | true    | Hide secrets in logs             |
| `block_env_mutation`  | bool | true    | Prevent `export`                 |
| `confirm_destructive` | bool | true    | Extra prompt for `rm`, `mv`, etc |

---

## Environment Variables

| Variable             | Scope    | Description      |
| -------------------- | -------- | ---------------- |
| `OPENAI_API_KEY`     | provider | OpenAI auth      |
| `SAMBANOVA_API_KEY`  | provider | Sambanova auth   |
| `GEMINI_API_KEY`     | provider | Gemini auth      |
| `HARPER_CONFIG_PATH` | global   | Config override  |
| `HARPER_DATA_DIR`    | global   | Storage override |

---

## Resolution Order

1. CLI flags
2. Environment variables
3. Config file
4. Defaults

---

## Validation Rules

| Rule               | Outcome                  |
| ------------------ | ------------------------ |
| Missing provider   | Startup failure          |
| Invalid TOML       | Startup failure          |
| Disallowed command | Rejected pre-exec        |
| Provider timeout   | Command generation fails |

No silent fallback occurs.

---

## `[exec_policy]`

Execution approval, sandbox presets, and bounded retry behavior for `run_command`.

| Key                      | Type             | Default          | Description |
| ------------------------ | ---------------- | ---------------- | ----------- |
| `approval_profile`       | string           | `allow_listed`   | `strict`, `allow_listed`, or `allow_all` |
| `sandbox_profile`        | string           | `disabled`       | `disabled`, `workspace`, or `networked_workspace` |
| `retry_max_attempts`     | integer          | `1`              | Max automatic retries for retry-safe failures |
| `retry_network_commands` | array of strings | `["curl","wget"]`| Network command classes eligible for bounded retry |
| `retry_write_commands`   | array of strings | `["mkdir","touch"]` | Write command classes eligible for bounded retry |

### `[exec_policy.sandbox]`

| Key              | Type             | Default | Description |
| ---------------- | ---------------- | ------- | ----------- |
| `allowed_dirs`   | array of strings | `[]`    | Readable roots inside the sandbox |
| `writable_dirs`  | array of strings | `[]`    | Writable roots inside the sandbox |
| `network_access` | bool             | profile | Whether sandboxed commands may reach the network |
| `readonly_home`  | bool             | profile | Whether `$HOME` is mounted read-only |

### Notes

- `allow_listed` still prompts when a command declares network access or writes outside configured writable roots.
- Retry behavior remains conservative: Harper uses declared intent plus configured command classes, not blind command replay.

---

## Minimal Example

```toml
[provider]
name = "openai"
model = "gpt-5.5"

[execution]
require_approval = true
allow_pipes = false

[ui]
theme = "minimal"
```

---

## Non-Goals

* Autonomous execution
* Background agents
* Remote command dispatch
* Self-modifying configuration
