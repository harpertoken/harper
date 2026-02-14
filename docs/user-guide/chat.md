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
| `/save` | Save current session to disk |
| `/load` | Load a saved session |
| `/audit` | View command history (e.g., `/audit 10`) |

### Using Commands

Simply type the command and press Enter. For example:

```
> /help
Available commands:
/exit, /quit - Exit the application
/help    - Show this message
/clear   - Clear current session
/save    - Save current session
/load    - Load a saved session
/audit   - View command history
```

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

Your chat sessions can be saved and loaded later. This feature allows you to:

- **Save Session**: Use `/save` to persist your conversation
- **Load Session**: Use `/load` to restore a previous conversation
- **Continue Later**: Pick up where you left off

Sessions are stored locally and include your entire conversation history with the AI.

### Command Logging

Harper logs all shell commands executed during your session. You can review these logs using the `/audit` command which shows:
- Command text
- Approval state (approved or rejected)
- Exit code
- Runtime duration
- stdout/stderr previews

This ensures every operation is traceable for security and debugging purposes.

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
