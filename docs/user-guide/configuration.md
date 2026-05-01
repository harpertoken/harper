# Configuration

Harper uses configuration files to customize its behavior, including API settings, model selection, and application preferences. This guide covers all configuration options.

Keep tracked config files as placeholders. Store real secrets such as API keys and Supabase credentials in `.env`, which Harper loads automatically at startup.

## Config File Location

Harper looks for configuration in the following locations (in order of priority):

1. `./config/local.toml` - Local project configuration
2. `~/.harper/config.toml` - User-level configuration
3. Default values (if no config found)

## Creating a Config File

Create a local config from the example:

```bash
mkdir -p config
cp config/local.example.toml config/local.toml
```

## Execution Policy

Harper separates command approval policy from sandbox policy under `[exec_policy]`. The same fields are also available in the TUI under `Settings -> Execution Policy`.

```toml
[exec_policy]
approval_profile = "allow_listed"   # strict | allow_listed | allow_all
execution_strategy = "auto"         # auto | grounded | deterministic | model
sandbox_profile = "workspace"       # disabled | workspace | networked_workspace
retry_max_attempts = 1
retry_network_commands = ["curl", "wget"]
retry_write_commands = ["mkdir", "touch"]

[ui]
header_widgets = ["model", "cwd", "strategy", "update"]

[exec_policy.sandbox]
allowed_dirs = ["."]
writable_dirs = ["./tmp", "./build"]
```

- `approval_profile` controls when Harper asks before running commands.
- `execution_strategy` controls whether Harper prefers direct grounded tool execution, deterministic-first grounding with model synthesis, tool-assisted behavior, or no deterministic shortcuts.
- `sandbox_profile` controls the default sandbox boundary.
- `retry_max_attempts` controls bounded automatic retries for retry-safe failures.
- `header_widgets` controls which status items appear in the chat header. You can edit that list from `Settings -> Execution Policy`, and saving the screen writes the selection back to `config/local.toml`.
- `Settings -> Execution Policy` also includes `Check for Updates`, which refreshes the release manifest and the `update` header widget without leaving the TUI.
- Direct self-update verifies both the published checksum and detached signature before replacing the local executable.
- `allowed_dirs` are readable roots.
- `writable_dirs` are writable roots.

Supported `header_widgets` values:

- `session`
- `plan`
- `agents`
- `web`
- `auth`
- `focus`
- `model`
- `cwd`
- `strategy`
- `approval`
- `update`
- `activity`

Under `allow_listed`, Harper still asks for approval when a command declares network access or writes outside configured writable roots, even if the command itself is allowlisted.

## Repo-aware routing

Harper now uses an explicit strategy-dependent control path for repo-aware work.

- `deterministic` prefers direct grounded tool execution for supported intents
- `grounded` prefers deterministic grounding first for routable repo questions, then allows model synthesis when needed
- `auto` remains tool-assisted and can still fall back to deterministic handling for supported prompts
- `model` disables deterministic shortcuts

Supported deterministic-style intents still include:

- direct operational facts such as `which branch am i on` or `which repo are we working on`
- direct file reads such as `read Cargo.toml`
- simple create/write prompts such as `create hello.rs with ...`
- direct command prompts such as `run git status`
- codebase prompts such as `where is X used`, `what calls X`, and `where is X defined`

For broader repo questions such as `tell me the codebase` or open-ended authoring prompts, Harper first gathers structured codebase or authoring context, then lets the model answer from that grounded context. If the model backend is unavailable and no deterministic fallback exists, Harper returns a clear assistant reply instead of a raw API error.

## API Configuration

### Setting Up OpenAI

```toml
[api]
key = "your-openai-api-key"
provider = "OpenAI"
```

### Setting Up Anthropic

```toml
[api]
key = "your-anthropic-api-key"
provider = "Anthropic"
```

### Setting Up Other Providers

```toml
[api]
key = "your-api-key"
provider = "ProviderName"
base_url = "https://api.example.com/v1"  # Optional: for custom endpoints
```

## Model Configuration

### Selecting Models

You can specify which models to use for different tasks:

```toml
[models]
default = "gpt-5.5"
code = "gpt-5.5"
chat = "gpt-5.5"
```

### Model Parameters

Configure model behavior with additional parameters:

```toml
[models.default]
temperature = 0.7
max_tokens = 2048
top_p = 1.0
frequency_penalty = 0.0
presence_penalty = 0.0
```

### Parameter Explanations

| Parameter | Description | Typical Range |
|-----------|-------------|---------------|
| temperature | Controls randomness | 0.0 - 2.0 |
| max_tokens | Maximum response length | 1 - 32000 |
| top_p | Nucleus sampling | 0.0 - 1.0 |
| frequency_penalty | Reduces repetition | -2.0 - 2.0 |
| presence_penalty | Encourages new topics | -2.0 - 2.0 |

## Application Settings

### General Settings

```toml
[app]
name = "Harper"
version = "1.0.0"
debug = false
verbose = false
```

### Session Settings

```toml
[session]
auto_save = true
save_interval = 300  # seconds
max_history = 1000
session_dir = "./sessions"
```

### UI Settings

```toml
[ui]
theme = "minimal"  # Options: minimal, cyberpunk, default, light, dark, github
```

## Environment Variables

You can also configure Harper using environment variables:

```bash
export HARPER_API_KEY="your-api-key"
export HARPER_PROVIDER="OpenAI"
export HARPER_MODEL="gpt-5.5"
export HARPER_DEBUG="false"
```

## Security

### API Key Best Practices

1. **Never commit API keys**: Add your config file to `.gitignore`
2. **Use environment variables**: Store sensitive values in env vars
3. **Rotate keys regularly**: Update your API keys periodically
4. **Use least privilege**: Use API keys with minimal required permissions

### Example with Environment Variables

```toml
[api]
key = "${HARPER_API_KEY}"  # Reads from environment variable
provider = "${HARPER_PROVIDER:-OpenAI}"  # With default value
```

## Multiple Profiles

You can create multiple configuration profiles:

```toml
[profile.development]
api.key = "dev-api-key"
debug = true

[profile.production]
api.key = "prod-api-key"
debug = false
```

Select a profile when starting Harper:

```bash
harper --profile development
```

## Troubleshooting

### Config Not Found

If Harper isn't finding your config:
- Check the file path is correct
- Verify the file is named `local.toml`
- Ensure proper TOML syntax

### Invalid Configuration

Common issues:
- Missing quotes around API keys
- Invalid TOML syntax (check for commas, brackets)
- Unsupported configuration options

### API Key Issues

- Verify the API key is correct
- Check the provider name is supported
- Ensure you have credits/quota available

## Advanced Configuration

### Custom Endpoints

```toml
[api]
provider = "OpenAI"
base_url = "https://api.openai.com/v1"
```

### Proxy Settings

```toml
[network]
proxy = "http://proxy.example.com:8080"
timeout = 30
```

### Logging

```toml
[logging]
level = "info"
file = "harper.log"
max_size = "10MB"
```
