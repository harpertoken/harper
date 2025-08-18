[![CI](https://github.com/bniladridas/harper/actions/workflows/ci.yml/badge.svg)](https://github.com/bniladridas/harper/actions/workflows/ci.yml)
[![Release](https://github.com/bniladridas/harper/actions/workflows/release.yml/badge.svg)](https://github.com/bniladridas/harper/actions/workflows/release.yml)

<div style="display: flex; align-items: flex-start; justify-content: space-between;">
  <p>
    Welcome to Harper AI Agent! A tool that connects to multiple AI providers, executes shell commands, and maintains conversation history, all on your system.
  </p>
  <img 
    src="https://github.com/user-attachments/assets/55c24e02-82ac-470f-b83b-1560e6b6fcd7" 
    alt="Harper AI Agent" 
    width="300" 
    style="margin-left: 20px;"
  />
</div>

# Harper AI Agent Guide

## Getting Started

### System Requirements
- Rust 1.70.0 or later
- Internet connection
- Supported OS: Linux, macOS, Windows (WSL2 recommended for Windows)

### Quick Installation
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build Harper
git clone https://github.com/bniladridas/harper.git
cd harper
make build

# Configure your API keys
cp env.example .env
# Edit .env with your preferred API key

# Run Harper
make run
````

## Core Features

### Multi-Provider AI Integration

| Provider      | Model                      | Best For                         |
| ------------- | -------------------------- | -------------------------------- |
| OpenAI        | GPT-4 Turbo                | General purpose, code generation |
| Sambanova     | Meta-Llama-3.2-1B-Instruct | Open-source alternative          |
| Google Gemini | Gemini 2.0 Flash           | Multimodal capabilities          |

### Command Execution

Execute shell commands directly from your conversation:

```
[RUN_COMMAND ls -la]
```

### Web Search

Perform web searches without leaving the interface:

```
[SEARCH: latest AI developments]
```

### Session Management

* Save and load conversation history
* Export conversations in multiple formats
* Persistent storage using SQLite

## Command Reference

### Basic Commands

```bash
make run       # Run Harper
make build     # Build release version
make test      # Run tests
```

### Development Commands

```bash
make fmt       # Format code
make lint      # Run linter
make doc       # Generate documentation
make clean     # Clean build artifacts
```

## Configuration

Create a `.env` file with your API keys:

```
# Choose one provider
OPENAI_API_KEY=your_openai_key
SAMBASTUDIO_API_KEY=your_sambanova_key
GEMINI_API_KEY=your_gemini_key
```

## Error Handling

* Syntax errors with exact locations
* Code quality checks via Clippy
* Test failures with detailed stack traces
* Security vulnerability scanning

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0 - See [LICENSE](LICENSE) for details.

## Support

* **Bug Reports**: Open an issue on GitHub
* **Feature Requests**: Start a discussion
* **Questions**: Check FAQ or open a discussion

## Community

* [GitHub Discussions](https://github.com/bniladridas/harper/discussions)
* [Discord Server](https://discord.gg/ENUnDfjA)
* [X](https://x.com/harper56889360)

## Philosophy

Harper empowers personal growth and self-understanding. By valuing our own capabilities, we create better tools and foster healthier relationships with technology.