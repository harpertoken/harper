# Chat Interface

Harper provides an interactive command-line chat interface that allows you to communicate with AI models. This guide covers all the features and commands available in the chat.

## Starting Harper

After installation, run Harper from your terminal:

```bash
./target/release/harper
```

You will see a welcome message followed by the chat prompt where you can type your messages.

## Chat Commands

Harper supports slash commands for various operations. All commands start with `/`:

| Command | Description |
|---------|-------------|
| `/exit` | Exit the application safely |
| `/quit` | Same as /exit |
| `/help` | Show available commands |
| `/clear` | Clear the current session |
| `/audit` | View command history (e.g., `/audit 10`) |
| `/strategy` | Show or change the execution strategy |
| `/strategy auto` | Switch the live chat session to `auto` |
| `/strategy grounded` | Switch the live chat session to `grounded` |
| `/strategy deterministic` | Switch the live chat session to `deterministic` |
| `/strategy model` | Switch the live chat session to `model` |
| `/update` | Show the cached update status in the TUI |
| `/update status` | Same as `/update` |
| `/update check` | Re-run the manifest check and refresh the header status |
| `/agents` | Show AGENTS context status for the current session |
| `/agents status` | Same as `/agents` |
| `/agents on` | Enable AGENTS context resolution in the TUI session |
| `/agents off` | Disable AGENTS context resolution in the TUI session |

### Using Commands

Simply type the command and press Enter. For example:

```
> /help
Available commands:
/exit, /quit - Exit the application
/help    - Show this message
/clear   - Clear current session
/audit   - View command history
/strategy - Show current execution strategy
/update - Show or refresh update status
/agents  - Show AGENTS context status
```

Typing `/` in the TUI opens the slash-command list. Use `↑` / `↓` or `Tab` to move through suggestions, then keep typing or submit the selected command from the normal message input.

`/update` shows the current cached update status from the header widget. Use `/update check` to re-run the manifest-backed update check on demand. The same refresh is also available from `Settings -> Execution Policy -> Updates`. Harper now checks the default GitHub release manifest automatically, and `HARPER_UPDATE_MANIFEST_URL` can override that source when needed. Published direct-install artifacts are verified with both a SHA-256 checksum and a detached signature before Harper replaces the local binary.

## Chat Features

### Multi-line Input

Harper supports multi-line input for longer messages. To enter multi-line mode, start your message with a blank line or use the enter key at the end of each line. When you want to send the message, press Enter twice or use a specific ending marker.

This is useful when you need to:
- Provide long code snippets
- Write detailed prompts
- Include multiple paragraphs

### Command History

Harper maintains a history of your commands within the session. Use the up and down arrow keys to navigate through previous commands. This makes it easy to:
- Repeat previous queries with modifications
- Review what you've asked before
- Quickly access frequently used commands

### Session Persistence

Harper keeps local sessions in its built-in session store. Use the Home screen, History screen, export flow, and session preview flow to revisit previous conversations rather than relying on ad hoc chat commands.

### Command Logging

Harper logs all shell commands executed during your session. You can review these logs using the `/audit` command which shows:
- Command text
- Approval state (approved or rejected)
- Exit code
- Runtime duration
- stdout/stderr previews

This ensures every operation is traceable for security and debugging purposes.

### Strategy-aware routing

Execution strategy changes how Harper chooses between deterministic tools and model-backed reasoning:

- `deterministic` prefers direct grounded tool execution for supported intents
- `grounded` prefers deterministic grounding first for routable repo questions, then allows model synthesis when needed
- `auto` remains tool-assisted and can still fall back to deterministic handling for supported prompts
- `model` disables deterministic shortcuts

For fast verification without the TUI, use `harper-batch`:

```bash
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic --prompt "where is execution strategy used in this repo"
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic --prompt "run the git status" --prompt "run that"
```

That prints the selected strategy, task mode, routed deterministic intent, normalized command when one exists, runtime activity, and the final assistant reply.

## Sending Messages

### Basic Message

Type your message and press Enter to send it to the AI. The response will appear below your message.

### Sending Code

When sending code, you can format it in markdown code blocks for better readability:

```
Here's a function I need help with:

```rust
fn main() {
    println!("Hello, World!");
}
```
```

### Interrupting Responses

If you need to stop a long-running response, press Ctrl+C to interrupt the AI.

## Tips and Best Practices

1. **Be Specific**: The more details you provide, the better the AI can help you.

2. **Use Context**: Reference previous messages to maintain continuity in the conversation.

3. **Save Important Sessions**: Save sessions that contain valuable information or complex problem-solving.

4. **Review History**: Use the audit feature to review what commands have been executed.

5. **Clear When Needed**: Use `/clear` to start fresh if the conversation becomes too long or confusing.
