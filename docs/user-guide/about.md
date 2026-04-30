# About Harper Binary

This document explains how Harper's binary works and how to run it.

## Binary Overview

Harper is a Rust application with multiple binaries available in the workspace. The main binary provides an interactive AI chat interface.

## Running Harper

### Using Cargo

The recommended way to run Harper from the workspace root:

```bash
cargo run -p harper-ui --bin harper
```

Or you can run from within the harper-ui directory:

```bash
cd lib/harper-ui
cargo run
```

### Using the Built Binary

After building:

```bash
# Debug build
./target/debug/harper

# Release build
./target/release/harper
```

## Available Binaries

### harper

The main interactive chat interface. This is what you use for:
- Chatting with AI models
- Processing clipboard content
- Session management
- Running commands with user approval

### harper-batch

Shell-facing chat harness for non-TUI debugging and scripted runs:

```bash
cargo run -p harper-ui --bin harper-batch -- --prompt "..."
```

Use this for:
- Verifying natural-language behavior without opening the TUI
- Comparing `auto`, `grounded`, `deterministic`, and `model` strategy routing
- Running repeatable prompt sequences in a single session
- Capturing runtime activity and command output in shell or JSON form

Supported options:

- `--prompt <text>` repeatable prompt input
- `--strategy <mode>` where mode is `auto`, `grounded`, `deterministic`, or `model`
- `--json` machine-readable output
- `--web` enable web search for the session

Examples:

```bash
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic --prompt "where is execution strategy used in this repo"
```

```bash
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic \
  --prompt "run the git status" \
  --prompt "run that"
```

```bash
printf 'hi\nwhat is the difference between grounded and deterministic\n' \
  | cargo run -p harper-ui --bin harper-batch -- --strategy auto
```

The output includes:

- routing summary
- runtime activity stages
- command output when a tool executes
- final assistant response

## How It Works

### Entry Point

1. **Cargo** finds the binary defined in `lib/harper-ui/Cargo.toml`:
   ```toml
   [[bin]]
   name = "harper"
   path = "src/main.rs"
   ```

2. **main.rs** initializes the application:
   - Loads configuration
   - Sets up the TUI (Terminal User Interface)
   - Connects to the configured AI provider
   - Starts the chat loop

3. **The Application** runs an event loop:
   - Reads user input
   - Sends to AI model
   - Displays response
   - Handles commands (like `/help`, `/strategy`, and `/agents`)

### Architecture

```
┌─────────────────────────────────────┐
│         harper binary               │
├─────────────────────────────────────┤
│  lib/harper-ui (TUI)               │
│  - User interface                   │
│  - Command parsing                  │
│  - Session management               │
├─────────────────────────────────────┤
│  lib/harper-core (Core)            │
│  - AI integration                   │
│  - Memory/Persistence               │
│  - Tools and plugins                │
└─────────────────────────────────────┘
```

## Build Options

### Debug Build

```bash
cargo build
# Slower runtime, faster compilation, larger binary
./target/debug/harper
```

### Release Build

```bash
cargo build --release
# Optimized, smaller binary
./target/release/harper
```

### Custom Profiles

```bash
cargo build --profile dist
# Maximum optimization with LTO
```

## Command Line Arguments

Run with help to see all options:

```bash
cargo run -- --help
```

Common arguments:
- `--config <path>` - Custom config file path
- `--session <name>` - Load specific session
- `--debug` - Enable debug mode

## Requirements

To run Harper you need:
- Rust 1.85.0 or later
- An API key for your chosen AI provider
- Terminal with sufficient colors support

## Troubleshooting

If Harper won't start:
- Check your API key is configured
- Verify your terminal supports colors
- Try running with debug mode enabled
- Check the troubleshooting guide

## See Also

- [Installation Guide](installation.md)
- [Configuration](configuration.md)
- [Chat Interface](chat.md)
