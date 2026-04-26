# Harper

Harper is a terminal assistant that translates what you type into shell commands. It shows you what it's about to run, asks for your approval, and keeps a log of everything so you can review it later. You can connect it to cloud AI services like OpenAI, Sambanova, Gemini, and Cerebras, or run it completely offline with Ollama.

<div align="center">
  <img src="https://raw.githubusercontent.com/harpertoken/harper/main/website/harper.png?v=2" width="600" alt="Harper interface" />
</div>

3+ themes (minimal, cyberpunk, etc). Set via `[ui]` in `config/local.toml`.

Getting started is simple. Clone the repo, copy the example env file, and run it:

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp .env.example .env
cargo run -p harper-ui --bin harper
```

Running in a sandbox (bubblewrap on Linux, sandbox-exec on macOS) isolates shell commands for safety.

Pick your AI provider in the config file. If you want offline mode, set up Ollama first, then point Harper to it.

When you're working on Harper itself, run `cargo fmt` to format, `cargo clippy --all-targets --all-features` to lint, and `cargo test` to test.

Check the docs for more on installation, configuration, and troubleshooting. Contributors should read CONTRIBUTING.md, and note that commercial use needs its own license.

Harper also includes an optional HTTP review API (port 8081) for editor integrations, plus a VS Code extension scaffold in `extensions/harper-review-vscode`.

Learn more at [`harper/server.html`](https://harpertoken.github.io/harper/server.html).
