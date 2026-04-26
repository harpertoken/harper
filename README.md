# Harper

Harper is a terminal assistant for code and shell work. It turns natural-language requests into explicit commands, shows exactly what it’s about to run, asks for approval when needed, and keeps a full audit trail.

The goal isn’t to replace the shell with chat. It’s to make command-driven work faster while keeping control visible.

<div align="center">
  <img src="https://raw.githubusercontent.com/harpertoken/harper/main/website/harper.png?v=2" width="600" alt="Harper interface" style="border-radius: 14px;" />
</div>

Harper works with OpenAI, SambaNova, Gemini, and Cerebras, or fully offline using Ollama. It keeps session history, supports planner-style task tracking, and exposes an optional HTTP review API for editor integrations.

To get started, clone the repo, copy the env file, and run the TUI:

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
cp .env.example .env
cargo harper
```

You can use Harper to inspect files, patch code, run commands, and manage multi-step work without losing track of what’s happening. Commands stay explicit, risky actions can require approval, and every session keeps both an audit log and active plan state. If you want to plug it into an editor workflow, the HTTP review API is there for that.

Configuration is straightforward. Pick your model provider in `config/local.toml`, set UI options under `[ui]`, and point Harper to Ollama if you want offline mode. Sandboxed execution is supported on Linux (bubblewrap) and macOS (sandbox-exec), so commands can run in isolation when configured.

If you are working on Harper itself, the normal development loop is:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

The workspace is split across `harper-core`, `harper-ui`, and supporting crates, so most changes touch both core logic and the TUI.

Harper also includes an optional HTTP review API (port `8081`), a VS Code scaffold in `extensions/harper-review-vscode`, more docs at [`harper/server.html`](https://harpertoken.github.io/harper/server.html), and contributor guidance in `CONTRIBUTING.md`.
