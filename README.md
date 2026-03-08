<div align="center">
  <img src=".freeman/grok/sign.jpg" alt="Freeman Sign" width="200" />
</div>

---

# Harper

Harper is an AI-powered terminal assistant designed to bridge the gap between human intent and shell execution. It translates natural language descriptions into precise, syntax-correct CLI commands across various shells and operating systems.

By leveraging advanced Large Language Models, Harper allows developers to perform complex operations—from deep filesystem searches to multi-stage Git workflows—without context-switching to documentation or web searches.

```bash
# Input: "find all docker volumes older than 3 days and remove them"
# Harper: docker volume ls -q -f "driver=local" | xargs -r docker volume rm
```

---

## Core Capabilities

### Natural Language Translation
Describe your objective in plain English. Harper analyzes the request and generates the exact shell command required. It supports standard Unix utilities, version control systems, and container orchestration tools.

### State-Aware Context
The assistant maintains a session-based history. Users can issue follow-up prompts (e.g., "now do that for the staging branch") without restating the original parameters.

### Explicit Security Model
Harper operates on a "Review-First" architecture. No command is executed without explicit user approval via a dedicated security modal. The system filters for dangerous sequences and provides a clear preview of the proposed action.

### Provider Agility
Integrate with your preferred AI backend. Harper provides native support for:
* OpenAI (GPT-4o, GPT-4 Turbo)
* SambaNova
* Google Gemini
* Any OpenAI-compatible REST endpoint

---

## Technical Specifications

### Build Systems
The project maintains support for both Cargo and Bazel to accommodate different enterprise environments.

| Task | Cargo Command | Bazel Target |
| :--- | :--- | :--- |
| Build (Release) | `cargo build --release` | `bazel build //...` |
| Run Interface | `cargo run -p harper-ui` | `bazel run //:harper` |
| Execute Tests | `cargo test` | `bazel test //...` |

### Security Features
* **Command Audit Trail**: Access a complete log of approved and failed commands via the `/audit` command.
* **Environment Isolation**: API credentials are never hardcoded and are managed strictly through environment variables.
* **Input Sanitization**: Proactive filtering of potentially malicious shell sequences and newline injections.

---

## Getting Started

### Prerequisites
* **Rust Toolchain**: 1.85.0 or higher
* **API Credentials**: A valid key for a supported AI provider
* **Terminal**: A terminal emulator with support for 24-bit color (TrueColor)

### Installation and Setup
```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp .env.example .env
# Edit .env to include your API_KEY and preferred PROVIDER
cargo run -p harper-ui --bin harper
```

---

## Contribution Policy

### Contributor License Agreement (CLA)
To maintain the legal integrity of the codebase, all contributors are required to sign a CLA. Pull requests will remain in a blocked state until the signature is verified by the automation bot.

* **Registry**: [harpertoken.github.io/cla.html](https://harpertoken.github.io/cla.html)
* **Support**: harpertoken@icloud.com

---

## Documentation
For detailed implementation details and advanced configuration, refer to the following:
* [Installation Guide](docs/user-guide/installation.md)
* [Configuration Reference](docs/CONFIG_REFERENCE.md)
* [Provider Setup](docs/user-guide/configuration.md)
* [Troubleshooting](docs/user-guide/troubleshooting.md)

---

## Licensing
Harper is dual-licensed under the **Apache License, Version 2.0** and the **MIT License**.

For entities requiring specialized terms or enterprise-grade support, please refer to the [COMMERCIAL_LICENSE](./COMMERCIAL_LICENSE) for further details.

<div align="center">
  <br />
  <sub>Built with precision by Harpertoken. &copy; 2026</sub>
</div>
