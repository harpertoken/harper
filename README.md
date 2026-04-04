# Harper

Harper is a terminal-first assistant platform that turns natural-language requests into safe shell actions. It keeps a full command log, prompts for approval before executing anything, and can run with cloud LLMs or entirely offline via Ollama.

![Harper interface](https://raw.githubusercontent.com/harpertoken/harper/main/website/harper.png)

## Quick Start

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp .env.example .env           # pick a provider below
cargo run -p harper-ui --bin harper
```

### Providers

```toml
# config/local.toml
[api]
provider  = "OpenAI" | "Sambanova" | "Gemini" | "Ollama"
api_key   = "..."              # leave empty for Ollama
base_url  = "https://..."      # or http://localhost:11434/api/chat
model_name = "gpt-4-turbo"     # e.g. llama3 for Ollama
```

For Ollama:
```bash
ollama serve &
ollama pull llama3
export OLLAMA_HOST=http://localhost:11434
export OLLAMA_MODEL=llama3
```

## Commands

| Task  | Command |
| --- | --- |
| Format | `cargo fmt` |
| Lint | `cargo clippy --all-targets --all-features` |
| Tests | `cargo test` |
| UI | `cargo run -p harper-ui --bin harper` |

## Docs

- [Configuration](docs/user-guide/configuration.md)
- [Installation guide](docs/user-guide/installation.md)
- [Troubleshooting](docs/user-guide/troubleshooting.md)

## Contributing & License

- CLA + guidelines: [CONTRIBUTING.md](CONTRIBUTING.md)
- Commercial terms only (see [COMMERCIAL_LICENSE](COMMERCIAL_LICENSE))
