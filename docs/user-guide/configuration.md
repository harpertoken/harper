# Configuration

Harper uses configuration files to customize its behavior, including API settings, model selection, and application preferences. This guide covers all configuration options.

## Config File Location

Harper looks for configuration in the following locations (in order of priority):

1. `./config/local.toml` - Local project configuration
2. `~/.harper/config.toml` - User-level configuration
3. Default values (if no config found)

## Creating a Config File

Create a `config/local.toml` file in the Harper directory:

```bash
mkdir -p config
touch config/local.toml
```

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
default = "gpt-4"
code = "gpt-4"
chat = "gpt-3.5-turbo"
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
theme = "dark"
color = true
prompt = "> "
show_timestamps = true
show_token_count = false
```

## Environment Variables

You can also configure Harper using environment variables:

```bash
export HARPER_API_KEY="your-api-key"
export HARPER_PROVIDER="OpenAI"
export HARPER_MODEL="gpt-4"
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
