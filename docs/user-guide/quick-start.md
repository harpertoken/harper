# Quick Start

Get up and running with Harper in just a few minutes. This guide walks you through the essential steps to start using Harper effectively.

## Before You Begin

Make sure you have:
- Installed Harper (see Installation guide)
- Created your configuration file with an API key
- A terminal or command prompt

## Starting Harper

1. Navigate to the Harper directory:
   ```bash
   cd harper
   ```

2. Run Harper:
   ```bash
   # Option 1: Using cargo (runs the binary directly)
   cargo run -p harper-ui --bin harper

   # Option 2: Using built binary (after building)
   ./target/release/harper
   ```

3. You should see a welcome message and the chat prompt:
   ```
   Welcome to Harper!

   Type your message and press Enter to start chatting.
   Type /help for a list of commands.

   >
   ```

## Your First Conversation

Try typing a simple message:

```
> Hello, what can you help me with?
```

The AI will respond based on your message. Try asking questions, requesting code help, or asking for explanations.

## Essential Commands

Here are the commands you'll use most often:

| Command | What It Does |
|---------|-------------|
| `/help` | Show all available commands |
| `/clear` | Start a new conversation |
| `/save` | Save your current session |
| `/load` | Load a saved session |
| `/exit` | Exit Harper |

## Basic Workflows

### Asking for Code Help

1. Start with a clear description of what you need
2. Include any relevant context or constraints
3. Ask specific questions

Example:
```
> Can you write a function in Rust that reads a file and counts the number of lines?
```

### Debugging Help

1. Copy the error message
2. Paste it in Harper
3. Ask what the error means

### Saving Work

1. After a productive session, type:
   ```
   > /save
   ```
2. Give your session a name when prompted
3. Later, use `/load` to continue

## Tips for Better Results

### Be Specific

Instead of:
```
> Help with code
```

Try:
```
> Write a Rust function that takes a vector of numbers and returns the average
```

### Provide Context

Include relevant information:
- What you're trying to accomplish
- Any errors you're seeing
- What you've already tried

### Use Follow-up Questions

Don't be afraid to ask clarifying questions:
```
> That worked, but how can I modify it to handle empty vectors?
```

## Next Steps

Now that you're familiar with the basics, explore more features:

- **Chat Interface**: Learn about multi-line input, history, and session management
- **Clipboard**: Use images and text from your clipboard
- **Configuration**: Customize Harper's behavior

## Getting Help

- Type `/help` in Harper for command reference
- Check the documentation for detailed guides
- Visit the GitHub repository for issues and discussions
